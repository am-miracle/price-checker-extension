// Unified product extractor - automatically detects site and extracts product data

import type { ProductMatchRequest } from "../type/item";
import { extractAmazonProduct, isAmazonProductPage } from "./amazon";
import { extractEbayProduct, isEbayProductPage } from "./ebay";
import { extractJumiaProduct, isJumiaProductPage } from "./jumia";
import { extractKongaProduct, isKongaProductPage } from "./konga";

export type SiteType = "amazon" | "ebay" | "jumia" | "konga" | "unknown";

// Detect which e-commerce site the user is currently on
export function detectCurrentSite(): SiteType {
  if (isAmazonProductPage()) return "amazon";
  if (isEbayProductPage()) return "ebay";
  if (isJumiaProductPage()) return "jumia";
  if (isKongaProductPage()) return "konga";
  return "unknown";
}

/**
 * Extract product information from the current page
 * Automatically detects the site and uses the appropriate extractor
 *
 * @returns Product data or null if extraction fails
 */
export function extractProductFromPage(): ProductMatchRequest | null {
  const site = detectCurrentSite();

  console.log("[Extractor] Detected site:", site);

  switch (site) {
    case "amazon":
      return extractAmazonProduct();
    case "ebay":
      return extractEbayProduct();
    case "jumia":
      return extractJumiaProduct();
    case "konga":
      return extractKongaProduct();
    case "unknown":
      console.warn("[Extractor] Unknown site, cannot extract product");
      return null;
    default:
      return null;
  }
}

// Check if current page is a supported product page

export function isSupportedProductPage(): boolean {
  return detectCurrentSite() !== "unknown";
}

// Re-export individual extractors for direct use if needed
export {
  extractAmazonProduct,
  extractEbayProduct,
  extractJumiaProduct,
  extractKongaProduct,
  isAmazonProductPage,
  isEbayProductPage,
  isJumiaProductPage,
  isKongaProductPage,
};
