import type { TrackedProduct, SitePrice } from "@/type/item";

const STORAGE_KEY = "tracked_products";
const CURRENCY_PREFERENCE_KEY = "currency_preference";

// Get all tracked products from Chrome storage
export async function getTrackedProducts(): Promise<TrackedProduct[]> {
  try {
    const result = await chrome.storage.local.get(STORAGE_KEY);
    return result[STORAGE_KEY] || [];
  } catch (error) {
    console.error("Error getting tracked products:", error);
    return [];
  }
}

// Save tracked products to Chrome storage
async function saveTrackedProducts(products: TrackedProduct[]): Promise<void> {
  try {
    await chrome.storage.local.set({ [STORAGE_KEY]: products });
  } catch (error) {
    console.error("Error saving tracked products:", error);
    throw error;
  }
}

// Generate unique ID for a product
function generateProductId(site: string, url: string): string {
  return `${site.toLowerCase()}-${btoa(url).slice(0, 20)}`;
}

export async function trackProduct(
  product: SitePrice,
  url: string,
): Promise<TrackedProduct> {
  const products = await getTrackedProducts();
  const id = generateProductId(product.site, url);

  // Check if already tracked
  const existing = products.find((p) => p.id === id);
  if (existing) {
    return existing;
  }

  const now = Date.now();
  const trackedProduct: TrackedProduct = {
    id,
    title: product.title,
    current_price: Number(product.price),
    currency: product.currency,
    site: product.site,
    url,
    image: product.image,
    tracked_at: now,
    last_checked: now,
    price_history: [
      {
        price: Number(product.price),
        timestamp: now,
      },
    ],
  };

  products.push(trackedProduct);
  await saveTrackedProducts(products);

  return trackedProduct;
}

export async function untrackProduct(productId: string): Promise<void> {
  const products = await getTrackedProducts();
  const filtered = products.filter((p) => p.id !== productId);
  await saveTrackedProducts(filtered);
}

// Check if a product is tracked
export async function isProductTracked(
  site: string,
  url: string,
): Promise<boolean> {
  const products = await getTrackedProducts();
  const id = generateProductId(site, url);
  return products.some((p) => p.id === id);
}

// Update tracked product price
export async function updateProductPrice(
  productId: string,
  newPrice: number,
): Promise<TrackedProduct | null> {
  const products = await getTrackedProducts();
  const product = products.find((p) => p.id === productId);

  if (!product) {
    return null;
  }

  const now = Date.now();

  // Only add to history if price changed
  if (product.current_price !== newPrice) {
    product.price_history.push({
      price: newPrice,
      timestamp: now,
    });

    // Keep only last 30 price points to avoid storage bloat
    if (product.price_history.length > 30) {
      product.price_history = product.price_history.slice(-30);
    }
  }

  product.current_price = newPrice;
  product.last_checked = now;

  await saveTrackedProducts(products);
  return product;
}

// Get price change for a tracked product
export function getPriceChange(product: TrackedProduct): {
  status: "increased" | "decreased" | "same";
  change_amount: number;
  change_percentage: number;
} {
  if (product.price_history.length < 2) {
    return {
      status: "same",
      change_amount: 0,
      change_percentage: 0,
    };
  }

  const originalPrice = product.price_history[0].price;
  const currentPrice = product.current_price;
  const change_amount = currentPrice - originalPrice;
  const change_percentage = (change_amount / originalPrice) * 100;

  let status: "increased" | "decreased" | "same" = "same";
  if (change_amount > 0) {
    status = "increased";
  } else if (change_amount < 0) {
    status = "decreased";
  }

  return {
    status,
    change_amount: Math.abs(change_amount),
    change_percentage: Math.abs(change_percentage),
  };
}

// Get user's currency preference
export async function getCurrencyPreference(): Promise<string> {
  try {
    const result = await chrome.storage.local.get(CURRENCY_PREFERENCE_KEY);
    return result[CURRENCY_PREFERENCE_KEY] || "USD"; // Default to USD
  } catch (error) {
    console.error("Error getting currency preference:", error);
    return "USD";
  }
}

// Save user's currency preference
export async function setCurrencyPreference(currency: string): Promise<void> {
  try {
    await chrome.storage.local.set({ [CURRENCY_PREFERENCE_KEY]: currency });

    // Notify background script to clear cache when currency changes
    chrome.runtime.sendMessage({ action: "clearCache" }).catch(() => {
      // Ignore errors if background script isn't ready
    });
  } catch (error) {
    console.error("Error saving currency preference:", error);
    throw error;
  }
}
