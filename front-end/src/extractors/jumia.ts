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
 * Extract product information from Jumia product pages
 * Supports: jumia.com.ng, jumia.com, jumia.co.ke, etc.
 */
export function extractJumiaProduct(): ProductMatchRequest | null {
  try {
    console.log("[Jumia Extractor] Starting extraction...");

    // Extract title
    const titleElement = querySelector([
      "h1.-fs20.-pts.-pbxs",
      ".name",
      "h1.title",
      ".product-title",
    ]);
    const rawTitle = getTextContent(titleElement);

    if (!rawTitle) {
      console.warn("[Jumia Extractor] Could not find product title");
      return null;
    }

    // Shorten title to maximum 7 words
    const title = shortenTitle(rawTitle) || rawTitle;

    console.log("[Jumia Extractor] Title:", title);

    // Extract price
    const priceElement = querySelector([
      ".-b.-ltr.-tal.-fs24.-prxs",
      ".price-box .price",
      ".-paxs",
      ".special-price",
    ]);
    const priceText = getTextContent(priceElement);
    const price = parsePrice(priceText);
    console.log("[Jumia Extractor] Price:", price, "from text:", priceText);

    // Detect currency (Jumia often uses local currencies)
    const currency = detectCurrency(priceText);

    // Extract brand
    const brandElement = querySelector([
      ".-fs14.-pvxs a",
      ".brand a",
      ".-m a.-pl0",
      ".subtitle a",
    ]);
    const brandRaw = getTextContent(brandElement);
    const brand = cleanBrandName(brandRaw);
    console.log("[Jumia Extractor] Brand:", brand);

    // Extract model/SKU from product details
    let modelNumber: string | undefined;
    let sku: string | undefined;

    const detailRows = document.querySelectorAll(
      ".-hr.-pvs li, .product-details li",
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

    // Alternative: Extract from key specifications table
    const specRows = document.querySelectorAll(
      ".-hr.-pvs article, .specifications-list li",
    );
    specRows.forEach((row) => {
      const label = row
        .querySelector(".-tal")
        ?.textContent?.toLowerCase()
        .trim();
      const value = row.querySelector(".-fw4")?.textContent?.trim();

      if (!label || !value) return;

      if (label.includes("model")) {
        modelNumber = value;
      } else if (label.includes("sku")) {
        sku = value;
      }
    });

    console.log("[Jumia Extractor] Model Number:", modelNumber, "SKU:", sku);

    // Extract specifications (color, size, etc.)
    const specifications: Record<string, string> = {};

    specRows.forEach((row) => {
      const label = row.querySelector(".-tal")?.textContent?.trim();
      const value = row.querySelector(".-fw4")?.textContent?.trim();

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

    console.log("[Jumia Extractor] Specifications:", specifications);

    // Extract product image
    // Based on the provided HTML structure: <img data-src="..." src="..." class="-fw -fh" ...>
    const imageElement = document.querySelector<HTMLImageElement>(
      ".-fw.-fh, img[data-src], .img-c img, .-fh img, .image-viewer img",
    );
    const image = imageElement?.dataset?.src || imageElement?.src || undefined;

    console.log("[Jumia Extractor] Image:", image);

    const productData: ProductMatchRequest = {
      title,
      current_price: price,
      currency,
      current_site: "jumia",
      url: window.location.href,
      image,
      identifiers: {
        brand,
        model_number: modelNumber,
        specifications:
          Object.keys(specifications).length > 0 ? specifications : undefined,
      },
    };

    console.log("[Jumia Extractor] Extraction successful:", productData);
    return productData;
  } catch (error) {
    console.error("[Jumia Extractor] Extraction failed:", error);
    return null;
  }
}

// Check if current page is a Jumia product page
export function isJumiaProductPage(): boolean {
  const hostname = window.location.hostname;
  const pathname = window.location.pathname;

  return hostname.includes("jumia.com") && pathname.includes(".html");
}
