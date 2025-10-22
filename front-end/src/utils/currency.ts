// Currency symbols
export const CURRENCY_SYMBOLS: Record<string, string> = {
  USD: "$",
  EUR: "€",
  GBP: "£",
  NGN: "₦",
  INR: "₹",
  CAD: "C$",
  AUD: "A$",
  JPY: "¥",
};

// Fallback exchange rates (used if API fails)
const FALLBACK_RATES: Record<string, number> = {
  USD: 1,
  EUR: 0.92,
  GBP: 0.79,
  NGN: 1500,
  INR: 83,
  CAD: 1.36,
  AUD: 1.52,
  JPY: 149,
};

// Cache for exchange rates
interface ExchangeRateCache {
  rates: Record<string, number>;
  timestamp: number;
}

const CACHE_DURATION = 60 * 60 * 1000; // 1 hour
const EXCHANGE_RATE_API = "https://api.exchangerate-api.com/v4/latest/USD";

let cachedRates: ExchangeRateCache | null = null;

// Fetch live exchange rates from API
async function fetchExchangeRates(): Promise<Record<string, number>> {
  try {
    // Check cache first
    if (cachedRates && Date.now() - cachedRates.timestamp < CACHE_DURATION) {
      return cachedRates.rates;
    }

    const response = await fetch(EXCHANGE_RATE_API);
    if (!response.ok) {
      throw new Error("Failed to fetch exchange rates");
    }

    const data = await response.json();
    const rates = data.rates as Record<string, number>;

    // Cache the rates
    cachedRates = {
      rates,
      timestamp: Date.now(),
    };

    console.log("[Currency] Fetched live exchange rates:", rates);
    return rates;
  } catch (error) {
    console.error(
      "[Currency] Failed to fetch exchange rates, using fallback:",
      error,
    );
    return FALLBACK_RATES;
  }
}

/**
 * Get current exchange rates (cached or fresh)
 */
export async function getExchangeRates(): Promise<Record<string, number>> {
  return await fetchExchangeRates();
}

/**
 * Convert price from one currency to another
 * All conversions go through USD as the base currency
 */
export async function convertCurrency(
  amount: number,
  fromCurrency: string,
  toCurrency: string,
): Promise<number> {
  if (fromCurrency === toCurrency) {
    return amount;
  }

  const rates = await getExchangeRates();
  const fromRate = rates[fromCurrency] || 1;
  const toRate = rates[toCurrency] || 1;

  // Convert to USD first, then to target currency
  const usdAmount = amount / fromRate;
  const convertedAmount = usdAmount * toRate;

  return convertedAmount;
}

/**
 * Convert price from one currency to another (synchronous, uses cached rates)
 */
export function convertCurrencySync(
  amount: number,
  fromCurrency: string,
  toCurrency: string,
  rates: Record<string, number>,
): number {
  if (fromCurrency === toCurrency) {
    return amount;
  }

  const fromRate = rates[fromCurrency] || 1;
  const toRate = rates[toCurrency] || 1;

  // Convert to USD first, then to target currency
  const usdAmount = amount / fromRate;
  const convertedAmount = usdAmount * toRate;

  return convertedAmount;
}

/**
 * Normalize price to USD (base currency for comparison)
 */
export async function normalizeToUSD(
  amount: number,
  currency: string,
): Promise<number> {
  const rates = await getExchangeRates();
  const rate = rates[currency] || 1;
  return amount / rate;
}

/**
 * Convert USD to target currency
 */
export async function convertFromUSD(
  usdAmount: number,
  targetCurrency: string,
): Promise<number> {
  const rates = await getExchangeRates();
  const rate = rates[targetCurrency] || 1;
  return usdAmount * rate;
}

/**
 * Convert USD to target currency (synchronous, uses provided rates)
 */
export function convertFromUSDSync(
  usdAmount: number,
  targetCurrency: string,
  rates: Record<string, number>,
): number {
  const rate = rates[targetCurrency] || 1;
  return usdAmount * rate;
}

/**
 * Format price with K for thousands, M for millions
 */
export function formatPrice(amount: number, decimals: number = 2): string {
  // Validate input
  if (typeof amount !== "number" || isNaN(amount)) {
    console.error("[Currency] formatPrice received invalid amount:", amount);
    return "0.00";
  }

  const absAmount = Math.abs(amount);

  if (absAmount >= 1_000_000) {
    // Use M for millions
    return (amount / 1_000_000).toFixed(decimals) + "M";
  } else if (absAmount >= 1_000) {
    // Use K for thousands
    return (amount / 1_000).toFixed(decimals) + "K";
  } else {
    // Regular number
    return amount.toFixed(decimals);
  }
}

/**
 * Format price with currency symbol and appropriate formatting
 */
export function formatPriceWithCurrency(
  amount: number,
  currency: string,
  useCompactFormat: boolean = false,
): string {
  const symbol = CURRENCY_SYMBOLS[currency] || currency;

  if (useCompactFormat) {
    return `${symbol}${formatPrice(amount)}`;
  } else {
    return `${symbol}${amount.toLocaleString(undefined, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })}`;
  }
}

/**
 * Get currency symbol for a currency code
 */
export function getCurrencySymbol(currency: string): string {
  return CURRENCY_SYMBOLS[currency] || currency;
}
