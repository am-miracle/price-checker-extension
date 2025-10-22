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
 * Extract product information from eBay product pages
 * Supports: ebay.com, ebay.co.uk, etc.
 */
export function extractEbayProduct(): ProductMatchRequest | null {
  try {
    console.log("[eBay Extractor] Starting extraction...");

    // Extract title
    const titleElement = querySelector([
      ".x-item-title__mainTitle",
      ".it-ttl",
      "#itemTitle",
      "h1.it-ttl",
    ]);
    const rawTitle = getTextContent(titleElement);

    if (!rawTitle) {
      console.warn("[eBay Extractor] Could not find product title");
      return null;
    }

    const title = shortenTitle(rawTitle) || rawTitle;

    console.log("[eBay Extractor] Title:", title);

    // Extract price
    const priceElement = querySelector([
      ".x-price-primary .ux-textspans",
      ".notranslate.vi-VR-cvipPrice",
      "#prcIsum",
      ".ux-textspans.ux-textspans--BOLD",
    ]);
    const priceText = getTextContent(priceElement);
    const price = parsePrice(priceText);
    console.log("[eBay Extractor] Price:", price, "from text:", priceText);

    // Detect currency
    const currency = detectCurrency(priceText);

    // Extract eBay Item ID from URL
    const itemIdFromURL = window.location.pathname.match(/\/itm\/(\d+)/)?.[1];
    const itemIdFromQuery = new URLSearchParams(window.location.search).get(
      "item",
    );
    const ebayItemId = itemIdFromURL || itemIdFromQuery || undefined;
    console.log("[eBay Extractor] eBay Item ID:", ebayItemId);

    // Extract brand from specifications
    let brand: string | undefined;
    let modelNumber: string | undefined;
    let upc: string | undefined;
    let ean: string | undefined;
    let mpn: string | undefined;

    // Try to extract from product specifications table
    const specLabels = document.querySelectorAll(".ux-labels-values__labels");
    const specValues = document.querySelectorAll(".ux-labels-values__values");

    specLabels.forEach((label, index) => {
      const labelText = label.textContent?.toLowerCase().trim() || "";
      const valueText = specValues[index]?.textContent?.trim();

      if (!valueText) return;

      if (labelText.includes("brand")) {
        brand = cleanBrandName(valueText);
      } else if (labelText.includes("model")) {
        modelNumber = valueText;
      } else if (labelText.includes("upc")) {
        upc = valueText;
      } else if (labelText.includes("ean")) {
        ean = valueText;
      } else if (
        labelText.includes("mpn") ||
        labelText.includes("manufacturer part")
      ) {
        mpn = valueText;
      }
    });

    // Fallback: Try old-style product details
    if (!brand || !modelNumber) {
      const detailRows = document.querySelectorAll(
        ".itemAttr tr, #viTabs_0_panel tr",
      );
      detailRows.forEach((row) => {
        const labelCell = row.querySelector(".attrLabels, td:first-child");
        const valueCell = row.querySelector("td:last-child");

        const labelText = labelCell?.textContent?.toLowerCase().trim() || "";
        const valueText = valueCell?.textContent?.trim();

        if (!valueText) return;

        if (!brand && labelText.includes("brand")) {
          brand = cleanBrandName(valueText);
        }
        if (!modelNumber && labelText.includes("model")) {
          modelNumber = valueText;
        }
        if (!upc && labelText.includes("upc")) {
          upc = valueText;
        }
        if (!ean && labelText.includes("ean")) {
          ean = valueText;
        }
      });
    }

    console.log("[eBay Extractor] Brand:", brand);
    console.log("[eBay Extractor] Model Number:", modelNumber);
    console.log("[eBay Extractor] UPC:", upc, "EAN:", ean, "MPN:", mpn);

    // Extract specifications (condition, color, etc.)
    const specifications: Record<string, string> = {};

    // Condition
    const conditionElement = querySelector([
      ".x-item-condition-text",
      ".vi-acc-del-range",
    ]);
    const condition = getTextContent(conditionElement);
    if (condition) specifications.condition = condition;

    console.log("[eBay Extractor] Specifications:", specifications);

    // Extract product image
    const imageElement = document.querySelector<HTMLImageElement>(
      'img[data-zoom-src], img[alt*="Picture"], #icImg, .ux-image-carousel-item img, .img-main img',
    );
    const image =
      imageElement?.dataset?.zoomSrc || imageElement?.src || undefined;

    console.log("[eBay Extractor] Image:", image);

    const productData: ProductMatchRequest = {
      title,
      current_price: price,
      currency,
      current_site: "ebay",
      url: window.location.href,
      image,
      identifiers: {
        ebay_item_id: ebayItemId,
        brand,
        model_number: modelNumber,
        upc,
        ean,
        mpn,
        specifications:
          Object.keys(specifications).length > 0 ? specifications : undefined,
      },
    };

    console.log("[eBay Extractor] Extraction successful:", productData);
    return productData;
  } catch (error) {
    console.error("[eBay Extractor] Extraction failed:", error);
    return null;
  }
}

//Check if current page is an eBay product page

export function isEbayProductPage(): boolean {
  const hostname = window.location.hostname;
  const pathname = window.location.pathname;

  return (
    hostname.includes("ebay.com") &&
    (pathname.includes("/itm/") || pathname.includes("/p/"))
  );
}
