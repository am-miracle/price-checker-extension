import type { ProductMatchRequest } from "../type/item";
import {
  parsePrice,
  detectCurrency,
  cleanBrandName,
  getTextContent,
  querySelector,
  shortenTitle,
} from "./utils";

/**
 * Extract product information from Konga product pages
 * Supports: konga.com
 */
export function extractKongaProduct(): ProductMatchRequest | null {
  try {
    console.log("[Konga Extractor] Starting extraction...");

    // Extract title
    const titleElement = querySelector([
      "h1",
      ".product-name",
      ".product-title",
      "h1.title",
      ".product-details-name",
      "[class*='product'] h1",
      "[class*='title']",
    ]);
    const rawTitle = getTextContent(titleElement);

    if (!rawTitle) {
      console.warn("[Konga Extractor] Could not find product title");
      return null;
    }

    // Shorten title to maximum 7 words
    const title = shortenTitle(rawTitle) || rawTitle;

    console.log("[Konga Extractor] Title:", title);

    // Extract price
    const priceElement = querySelector([
      "[class*='price']",
      ".price-box .price",
      ".product-price",
      ".current-price",
      ".special-price",
      "[data-price]",
    ]);
    const priceText = getTextContent(priceElement);
    const price = parsePrice(priceText);
    console.log("[Konga Extractor] Price:", price, "from text:", priceText);

    // Currency (Konga is Nigerian, uses Naira)
    const currency = detectCurrency(priceText, "konga.com");

    // Extract brand
    const brandElement = querySelector([
      ".product-brand",
      ".brand-name",
      'a[href*="/brand/"]',
    ]);
    const brandRaw = getTextContent(brandElement);
    const brand = cleanBrandName(brandRaw);
    console.log("[Konga Extractor] Brand:", brand);

    // Extract model/SKU from product details
    let modelNumber: string | undefined;
    let sku: string | undefined;

    const detailRows = document.querySelectorAll(
      ".product-details-list li, .specification-list li",
    );
    detailRows.forEach((row) => {
      const text = row.textContent?.trim() || "";
      const lowerText = text.toLowerCase();

      if (lowerText.includes("model:") || lowerText.includes("model number:")) {
        modelNumber = text.split(":")[1]?.trim();
      } else if (lowerText.includes("sku:")) {
        sku = text.split(":")[1]?.trim();
      }
    });

    // Alternative: Extract from specifications table
    const specRows = document.querySelectorAll(
      ".specifications-table tr, .product-info-table tr",
    );
    specRows.forEach((row) => {
      const labelCell = row.querySelector("td:first-child, th");
      const valueCell = row.querySelector("td:last-child");

      const label = labelCell?.textContent?.toLowerCase().trim();
      const value = valueCell?.textContent?.trim();

      if (!label || !value) return;

      if (label.includes("model")) {
        modelNumber = value;
      } else if (label.includes("sku")) {
        sku = value;
      }
    });

    console.log("[Konga Extractor] Model Number:", modelNumber, "SKU:", sku);

    // Extract specifications
    const specifications: Record<string, string> = {};

    specRows.forEach((row) => {
      const labelCell = row.querySelector("td:first-child, th");
      const valueCell = row.querySelector("td:last-child");

      const label = labelCell?.textContent?.trim();
      const value = valueCell?.textContent?.trim();

      if (label && value) {
        const key = label.toLowerCase().replace(/[^a-z0-9]/g, "_");
        if (key.includes("color") || key.includes("colour")) {
          specifications.color = value;
        } else if (key.includes("size")) {
          specifications.size = value;
        } else if (key.includes("storage") || key.includes("capacity")) {
          specifications.storage = value;
        }
      }
    });

    console.log("[Konga Extractor] Specifications:", specifications);

    // Extract product image
    // Based on the provided HTML structure: <img alt="..." sizes="..." srcset="..." src="..." ...>
    const imageElement = document.querySelector<HTMLImageElement>(
      "img[srcset], .product-image img, .main-image img, .product-gallery img",
    );

    // Try to get the highest quality image from srcset
    let image: string | undefined;

    if (imageElement?.srcset) {
      const srcsetParts = imageElement.srcset.split(",");
      // Get the last (highest resolution) image from srcset
      const lastSrcset = srcsetParts[srcsetParts.length - 1]?.trim();
      image = lastSrcset?.split(" ")[0]; // Extract URL from "url width" format
    }

    if (!image) {
      image = imageElement?.src || undefined;
    }

    console.log("[Konga Extractor] Image:", image);

    const productData: ProductMatchRequest = {
      title,
      current_price: price,
      currency,
      current_site: "konga",
      url: window.location.href,
      image,
      identifiers: {
        brand,
        model_number: modelNumber,
        specifications:
          Object.keys(specifications).length > 0 ? specifications : undefined,
      },
    };

    console.log("[Konga Extractor] Extraction successful:", productData);
    return productData;
  } catch (error) {
    console.error("[Konga Extractor] Extraction failed:", error);
    return null;
  }
}

// Check if current page is a Konga product page
export function isKongaProductPage(): boolean {
  const hostname = window.location.hostname;
  const pathname = window.location.pathname;

  return (
    hostname.includes("konga.com") &&
    (pathname.includes("/product/") || pathname.includes("/p/"))
  );
}
