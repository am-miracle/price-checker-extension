/**
 * Content script - Runs on e-commerce product pages
 * Extracts product information and communicates with popup/background
 */

import {
  extractProductFromPage,
  isSupportedProductPage,
  detectCurrentSite,
} from "./extractors";

console.log("[Content Script] Loaded on:", window.location.href);

// Message listener for communication with popup/background
chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  console.log("[Content Script] Received message:", message);

  if (message.action === "extractProduct") {
    handleExtractProduct(sendResponse);
    return true; // Keep channel open for async response
  }

  if (message.action === "checkSupported") {
    sendResponse({
      supported: isSupportedProductPage(),
      site: detectCurrentSite(),
    });
    return false;
  }

  return false;
});

/**
 * Handle product extraction request
 */
function handleExtractProduct(sendResponse: (response: any) => void) {
  try {
    if (!isSupportedProductPage()) {
      sendResponse({
        success: false,
        error: "Unsupported site",
        site: detectCurrentSite(),
      });
      return;
    }

    const productData = extractProductFromPage();

    if (!productData) {
      sendResponse({
        success: false,
        error: "Failed to extract product data",
        site: detectCurrentSite(),
      });
      return;
    }

    console.log("[Content Script] Extraction successful:", productData);

    sendResponse({
      success: true,
      data: productData,
      site: detectCurrentSite(),
    });
  } catch (error) {
    console.error("[Content Script] Extraction error:", error);
    sendResponse({
      success: false,
      error: error instanceof Error ? error.message : "Unknown error",
      site: detectCurrentSite(),
    });
  }
}

// Optional: Send notification when page loads if it's a supported site
if (isSupportedProductPage()) {
  console.log("[Content Script] Supported product page detected");

  // You can optionally badge the extension icon or send a notification
  chrome.runtime
    .sendMessage({
      action: "productPageDetected",
      site: detectCurrentSite(),
    })
    .catch(() => {
      // Ignore errors if background script isn't ready
    });
}
