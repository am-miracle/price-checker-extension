// Product identifiers for matching across sites
export interface ProductIdentifiers {
  upc?: string;
  ean?: string;
  gtin?: string;
  asin?: string;
  ebay_item_id?: string;
  mpn?: string; // Manufacturer Part Number
  model_number?: string;
  brand?: string;
  specifications?: Record<string, string>;
}

// Request body for POST /api/compare
export interface ProductMatchRequest {
  title: string;
  current_price?: number;
  currency?: string;
  current_site?: string;
  url?: string;
  image?: string;
  target_currency?: string;
  identifiers?: ProductIdentifiers;
}

// Individual price from a specific site
export interface SitePrice {
  site: string;
  title: string;
  price: number;
  currency: string;
  price_usd: number;
  price_converted?: number;
  target_currency?: string;
  link: string;
  image: string | null;
  match_confidence: number | null;
}

// API response from both GET and POST /api/compare
export interface PriceComparisonResult {
  best_deal: SitePrice | null;
  all_prices: SitePrice[];
}

// Legacy type for backward compatibility
export type Item = SitePrice;

// Tracked product with price history
export interface TrackedProduct {
  id: string; // Unique identifier (combination of site + product ID)
  title: string;
  current_price: number;
  currency: string;
  site: string;
  url: string;
  image: string | null;
  tracked_at: number; // Timestamp when first tracked
  last_checked: number; // Timestamp of last price check
  price_history: PricePoint[];
  identifiers?: ProductIdentifiers;
}

export interface PricePoint {
  price: number;
  timestamp: number;
}

// Price change status for tracked items
export interface PriceChangeStatus {
  product: TrackedProduct;
  status: "increased" | "decreased" | "same";
  change_amount?: number;
  change_percentage?: number;
}
