import type {
  ProductMatchRequest,
  PriceComparisonResult,
  SitePrice,
} from "./type/item";
import { getCurrencyPreference } from "./utils/storage";
import { getExchangeRates, convertFromUSDSync } from "./utils/currency";

const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL || "https://price-checker-extension.onrender.com/api/";
const CACHE_DURATION = 5 * 60 * 1000; // 5 minutes

interface CachedResult {
  data: PriceComparisonResult;
  timestamp: number;
}

// In-memory cache (persists while browser is open)
const cache = new Map<string, CachedResult>();

console.log("[Background] Service worker initialized");

// Listen for messages from popup or content script
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  console.log("[Background] Received message:", message);

  if (message.action === "comparePrice") {
    handleComparePriceRequest(message.productData, sendResponse);
    return true; // Keep channel open for async response
  }

  if (message.action === "productPageDetected") {
    // Update badge when on a product page
    updateBadge(sender.tab?.id, message.site);
    return false;
  }

  if (message.action === "clearCache") {
    // Clear cache when currency changes
    cache.clear();
    console.log("[Background] Cache cleared due to currency change");
    return false;
  }

  return false;
});

// Handle price comparison request
async function handleComparePriceRequest(
  productData: ProductMatchRequest,
  sendResponse: (response: any) => void,
) {
  try {
    console.log(
      "[Background] Handling price comparison for:",
      productData.title,
    );

    // Get user's currency preference
    const targetCurrency = await getCurrencyPreference();

    // Add target currency to product data
    const requestData = {
      ...productData,
      target_currency: targetCurrency,
    };

    // Check cache first
    const cacheKey = generateCacheKey(requestData);
    const cached = cache.get(cacheKey);

    if (cached && Date.now() - cached.timestamp < CACHE_DURATION) {
      console.log("[Background] Returning cached result");
      sendResponse({
        success: true,
        data: cached.data,
        cached: true,
      });
      return;
    }

    // Call API
    const response = await fetch(`${API_BASE_URL}/compare`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(requestData),
    });

    if (!response.ok) {
      throw new Error(
        `API returned ${response.status}: ${response.statusText}`,
      );
    }

    const data: PriceComparisonResult = await response.json();

    // Apply currency conversion to all prices using live exchange rates
    const convertedData = await applyUserCurrencyConversion(
      data,
      targetCurrency,
    );

    // Cache the result (with converted prices)
    cache.set(cacheKey, {
      data: convertedData,
      timestamp: Date.now(),
    });

    console.log("[Background] API call successful, results:", convertedData);

    sendResponse({
      success: true,
      data: convertedData,
      cached: false,
    });
  } catch (error) {
    console.error("[Background] API call failed:", error);

    sendResponse({
      success: false,
      error: error instanceof Error ? error.message : "Unknown error",
    });
  }
}

/**
 * Apply currency conversion to all prices in the result
 * Prices from API are normalized to USD, we convert to user's preference using live exchange rates
 */
async function applyUserCurrencyConversion(
  data: PriceComparisonResult,
  targetCurrency: string,
): Promise<PriceComparisonResult> {
  // If target currency is USD, no conversion needed (API already returns USD)
  if (targetCurrency === "USD") {
    return data;
  }

  // Fetch live exchange rates
  const rates = await getExchangeRates();

  // Convert all prices
  const convertedPrices: SitePrice[] = data.all_prices.map((price) => {
    const convertedAmount = convertFromUSDSync(
      price.price_usd,
      targetCurrency,
      rates,
    );
    return {
      ...price,
      price_converted: convertedAmount,
      target_currency: targetCurrency,
    };
  });

  // Convert best deal
  const convertedBestDeal = data.best_deal
    ? {
        ...data.best_deal,
        price_converted: convertFromUSDSync(
          data.best_deal.price_usd,
          targetCurrency,
          rates,
        ),
        target_currency: targetCurrency,
      }
    : null;

  return {
    all_prices: convertedPrices,
    best_deal: convertedBestDeal,
  };
}

// Generate cache key from product data
function generateCacheKey(productData: ProductMatchRequest): string {
  // Use title + identifiers for cache key
  const identifiers = productData.identifiers;
  const key = [
    productData.title,
    identifiers?.asin,
    identifiers?.upc,
    identifiers?.ean,
    identifiers?.ebay_item_id,
    productData.target_currency || "USD", // Include currency in cache key
  ]
    .filter(Boolean)
    .join("|");

  return key.toLowerCase();
}

// Update extension badge to show supported site
function updateBadge(tabId: number | undefined, site: string) {
  if (!tabId) return;

  const badges: Record<string, { text: string; color: string }> = {
    amazon: { text: "AMZ", color: "#FF9900" },
    ebay: { text: "BAY", color: "#E53238" },
    jumia: { text: "JUM", color: "#F68B24" },
    konga: { text: "KON", color: "#ED017F" },
  };

  const badge = badges[site];

  if (badge) {
    chrome.action.setBadgeText({ text: badge.text, tabId });
    chrome.action.setBadgeBackgroundColor({ color: badge.color, tabId });
  }
}

// Clear cache periodically (every 30 minutes)
setInterval(
  () => {
    const now = Date.now();
    let cleared = 0;

    for (const [key, value] of cache.entries()) {
      if (now - value.timestamp > CACHE_DURATION) {
        cache.delete(key);
        cleared++;
      }
    }

    if (cleared > 0) {
      console.log(`[Background] Cleared ${cleared} expired cache entries`);
    }
  },
  30 * 60 * 1000,
);
