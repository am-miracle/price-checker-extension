/**
 * Parse price from text containing currency symbols and formatting
 * Examples: "$999.99", "£299.00", "₦450,000.00"
 */
export function parsePrice(
  priceText: string | null | undefined,
): number | undefined {
  if (!priceText) return undefined;

  // Remove currency symbols and common formatting
  const cleaned = priceText
    .replace(/[$£€₦₹¥]/g, "") // Remove currency symbols
    .replace(/[,\s]/g, "") // Remove commas and spaces
    .trim();

  const price = parseFloat(cleaned);
  return isNaN(price) ? undefined : price;
}

// Detect currency from price text or domain

export function detectCurrency(priceText?: string, hostname?: string): string {
  const text = priceText || "";
  const domain = hostname || window.location.hostname;

  // Check for currency symbols first
  if (text.includes("$")) return "USD";
  if (text.includes("£")) return "GBP";
  if (text.includes("€")) return "EUR";
  if (text.includes("₦")) return "NGN";
  if (text.includes("₹")) return "INR";
  if (text.includes("¥")) return "JPY";
  if (text.includes("C$") || text.includes("CAD")) return "CAD";
  if (text.includes("A$") || text.includes("AUD")) return "AUD";

  // Fallback to domain-based detection
  if (domain.includes(".com")) return "USD";
  if (domain.includes(".co.uk")) return "GBP";
  if (
    domain.includes(".de") ||
    domain.includes(".fr") ||
    domain.includes(".es")
  )
    return "EUR";
  if (domain.includes(".ng") || domain.includes("jumia.com.ng")) return "NGN";
  if (domain.includes(".in")) return "INR";
  if (domain.includes(".jp")) return "JPY";
  if (domain.includes(".ca")) return "CAD";
  if (domain.includes(".au")) return "AUD";

  return "USD"; // Default fallback
}

export function cleanBrandName(
  brandText: string | null | undefined,
): string | undefined {
  if (!brandText) return undefined;

  let cleaned = brandText.trim();

  // Remove common prefixes
  cleaned = cleaned.replace(/^Visit the /i, "");
  cleaned = cleaned.replace(/^Brand:\s*/i, "");
  cleaned = cleaned.replace(/^By\s*/i, "");
  cleaned = cleaned.replace(/ Store$/i, "");
  cleaned = cleaned.replace(/ Official$/i, "");

  return cleaned.trim() || undefined;
}

// Extract text content from element with null safety
export function getTextContent(
  element: Element | null | undefined,
): string | undefined {
  const text = element?.textContent?.trim();
  if (!text) return undefined;

  // Remove zero-width characters, excessive whitespace, and normalize
  const cleaned = text
    .replace(/[\u200B-\u200D\uFEFF]/g, "") // Remove zero-width chars
    .replace(/\s+/g, " ") // Collapse multiple spaces
    .trim();

  return cleaned || undefined;
}

// Extract attribute from element with null safety
export function getAttribute(
  element: Element | null | undefined,
  attr: string,
): string | undefined {
  const value = element?.getAttribute(attr);
  return value ?? undefined;
}

// Find element by trying multiple selectors
export function querySelector(selectors: string[]): Element | null {
  for (const selector of selectors) {
    const element = document.querySelector(selector);
    if (element) return element;
  }
  return null;
}

// Extract value from meta tag
export function getMetaContent(property: string): string | undefined {
  const meta = document.querySelector(
    `meta[property="${property}"], meta[name="${property}"]`,
  );
  return getAttribute(meta, "content");
}

// Check if current URL matches a pattern
export function matchesURL(pattern: RegExp): boolean {
  return pattern.test(window.location.href);
}

// Extract from key-value table
export function extractFromTable(
  tableSelector: string,
  keyPattern: RegExp,
): string | undefined {
  const rows = document.querySelectorAll(tableSelector);

  for (const row of rows) {
    // Try table row format (th/td)
    const keyCell = row.querySelector("th, td:first-child, .a-span3, .label");
    const valueCell = row.querySelector("td:last-child, .a-span9, .value");

    let keyText = keyCell?.textContent?.trim().toLowerCase() || "";

    // For list items, the whole text might be "Key : Value"
    if (!valueCell && row.tagName === "LI") {
      const fullText = row.textContent || "";
      const parts = fullText.split(/[:\-]/);
      if (parts.length >= 2) {
        keyText = parts[0].trim().toLowerCase();
        const valueText = parts.slice(1).join(":").trim();

        if (keyPattern.test(keyText)) {
          return valueText;
        }
      }
    }

    if (keyPattern.test(keyText)) {
      const value = getTextContent(valueCell);
      if (value) {
        // Clean up common patterns like "Product : 123456" -> "123456"
        return value.replace(/^[:\-\s]+/, "").trim();
      }
    }
  }

  return undefined;
}

// Shorten title
export function shortenTitle(
  title: string | undefined,
  maxWords: number = 7,
): string | undefined {
  if (!title) return undefined;

  const words = title.trim().split(/\s+/);

  if (words.length <= maxWords) {
    return title.trim();
  }

  return words.slice(0, maxWords).join(" ");
}
