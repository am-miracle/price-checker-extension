import type { ProductMatchRequest } from "../type/item";
import {
  parsePrice,
  detectCurrency,
  cleanBrandName,
  getTextContent,
  querySelector,
  extractFromTable,
  shortenTitle,
} from "./utils";

/**
 * Extract product information from Amazon product pages
 * Supports: amazon.com, amazon.co.uk, etc.
 */
export function extractAmazonProduct(): ProductMatchRequest | null {
  try {
    console.log("[Amazon Extractor] Starting extraction...");

    // Extract title
    const titleElement = querySelector([
      "#productTitle",
      "#title",
      ".product-title",
    ]);
    const rawTitle = getTextContent(titleElement);

    if (!rawTitle) {
      console.warn("[Amazon Extractor] Could not find product title");
      return null;
    }

    const title = shortenTitle(rawTitle) || rawTitle;

    console.log("[Amazon Extractor] Title:", title);

    // Extract price
    const priceElement = querySelector([
      ".a-price .a-offscreen",
      "#priceblock_ourprice",
      "#priceblock_dealprice",
      ".a-price-whole",
    ]);
    const priceText = getTextContent(priceElement);
    const price = parsePrice(priceText);
    console.log("[Amazon Extractor] Price:", price, "from text:", priceText);

    // Detect currency
    const currency = detectCurrency(priceText);

    // Extract ASIN (Amazon Standard Identification Number)
    const asinFromURL =
      window.location.pathname.match(/\/dp\/([A-Z0-9]{10})/)?.[1];
    const asinInput = document.querySelector(
      'input[name="ASIN"]',
    ) as HTMLInputElement;
    const asinFromInput = asinInput?.value;
    const asin = asinFromURL || asinFromInput || undefined;
    console.log("[Amazon Extractor] ASIN:", asin);

    // Extract brand
    const brandElement = querySelector([
      "#bylineInfo",
      ".po-brand .po-break-word",
      "a#brand",
      '[data-feature-name="bylineInfo"]',
    ]);
    const brandRaw = getTextContent(brandElement);
    const brand = cleanBrandName(brandRaw);
    console.log("[Amazon Extractor] Brand:", brand);

    // Extract model number from product details
    const modelNumber =
      extractFromTable(
        "#productDetails_detailBullets_sections1 tr, .prodDetTable tr, #productDetails_techSpec_section_1 tr",
        /model|item model number/i,
      ) || extractFromTable("#detailBullets_feature_div li", /model number/i);

    console.log("[Amazon Extractor] Model Number:", modelNumber);

    // Extract UPC/EAN/GTIN from product details (try multiple locations)
    const upc =
      extractFromTable(
        "#productDetails_detailBullets_sections1 tr, .prodDetTable tr, #detailBullets_feature_div li, #productDetails_techSpec_section_1 tr",
        /^upc$|upc\s*:|universal product code/i,
      ) || extractFromTable("#detailBulletsWrapper_feature_div ul li", /upc/i);

    const ean = extractFromTable(
      "#productDetails_detailBullets_sections1 tr, .prodDetTable tr, #detailBullets_feature_div li, #productDetails_techSpec_section_1 tr",
      /^ean$|ean\s*:|european article number/i,
    );

    const gtin = extractFromTable(
      "#productDetails_detailBullets_sections1 tr, .prodDetTable tr, #detailBullets_feature_div li",
      /gtin|global trade item/i,
    );

    console.log("[Amazon Extractor] UPC:", upc, "EAN:", ean, "GTIN:", gtin);

    // Extract specifications (color, size, style, etc.)
    const specifications: Record<string, string> = {};

    const colorElement = document.querySelector(
      "#variation_color_name .selection",
    );
    const color = getTextContent(colorElement);
    if (color) specifications.color = color;

    // Size/Storage
    const sizeElement = document.querySelector(
      "#variation_size_name .selection",
    );
    const size = getTextContent(sizeElement);
    if (size) specifications.size = size;

    // Style
    const styleElement = document.querySelector(
      "#variation_style_name .selection",
    );
    const style = getTextContent(styleElement);
    if (style) specifications.style = style;

    console.log("[Amazon Extractor] Specifications:", specifications);

    // Extract product image
    const imageElement = document.querySelector<HTMLImageElement>(
      "#landingImage, .a-dynamic-image, #imgBlkFront, #main-image",
    );

    // Try to get highest quality image available
    let image: string | undefined;

    if (imageElement?.dataset?.aDynamicImage) {
      try {
        const imageData = JSON.parse(imageElement.dataset.aDynamicImage);
        // Get the URL with the highest resolution (last entry)
        const urls = Object.keys(imageData);
        image = urls[urls.length - 1];
      } catch (e) {
        console.warn(
          "[Amazon Extractor] Failed to parse dynamic image data:",
          e,
        );
      }
    }

    if (!image) {
      image = imageElement?.dataset?.oldHires || imageElement?.src || undefined;
    }

    console.log("[Amazon Extractor] Image:", image);

    const productData: ProductMatchRequest = {
      title,
      current_price: price,
      currency,
      current_site: "amazon",
      url: window.location.href,
      image,
      identifiers: {
        asin,
        brand,
        model_number: modelNumber,
        upc,
        ean,
        gtin,
        specifications:
          Object.keys(specifications).length > 0 ? specifications : undefined,
      },
    };

    console.log("[Amazon Extractor] Extraction successful:", productData);
    return productData;
  } catch (error) {
    console.error("[Amazon Extractor] Extraction failed:", error);
    return null;
  }
}

// Check if current page is an Amazon product page
export function isAmazonProductPage(): boolean {
  const hostname = window.location.hostname;
  const pathname = window.location.pathname;

  return (
    hostname.includes("amazon.com") &&
    (pathname.includes("/dp/") || pathname.includes("/gp/product/"))
  );
}
