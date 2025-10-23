import { useState, useEffect } from "react";
import type { PriceComparisonResult, ProductMatchRequest } from "@/type/item";

interface UsePriceComparisonState {
  data: PriceComparisonResult | null;
  loading: boolean;
  error: string | null;
  isSupported: boolean;
  currentSite: string | null;
  extractedProduct: ProductMatchRequest | null;
}

export function usePriceComparison() {
  const [state, setState] = useState<UsePriceComparisonState>({
    data: null,
    loading: true,
    error: null,
    isSupported: false,
    currentSite: null,
    extractedProduct: null,
  });

  useEffect(() => {
    checkCurrentPage();
  }, []);

  // Check if current page is a supported product page

  async function checkCurrentPage() {
    try {
      const [tab] = await chrome.tabs.query({
        active: true,
        currentWindow: true,
      });

      if (!tab?.id) {
        setState((prev) => ({
          ...prev,
          loading: false,
          error: "Unable to access current tab",
        }));
        return;
      }

      // Check if page is supported
      const response = await chrome.tabs.sendMessage(tab.id, {
        action: "checkSupported",
      });

      if (response.supported) {
        setState((prev) => ({
          ...prev,
          isSupported: true,
          currentSite: response.site,
        }));

        // Automatically extract product data
        await extractAndCompare(tab.id);
      } else {
        setState((prev) => ({
          ...prev,
          loading: false,
          isSupported: false,
          currentSite: response.site,
        }));
      }
    } catch (error) {
      console.error("Error checking page:", error);
      setState((prev) => ({
        ...prev,
        loading: false,
        error: error instanceof Error ? error.message : "Failed to check page",
      }));
    }
  }

  async function extractAndCompare(tabId: number) {
    setState((prev) => ({ ...prev, loading: true, error: null }));

    try {
      // Extract product from page
      const extractResponse = await chrome.tabs.sendMessage(tabId, {
        action: "extractProduct",
      });

      if (!extractResponse.success) {
        throw new Error(extractResponse.error || "Failed to extract product");
      }

      const productData: ProductMatchRequest = extractResponse.data;

      setState((prev) => ({ ...prev, extractedProduct: productData }));

      // Send to background script to compare prices
      const compareResponse = await chrome.runtime.sendMessage({
        action: "comparePrice",
        productData,
      });

      if (!compareResponse.success) {
        throw new Error(compareResponse.error || "Failed to compare prices");
      }

      console.log(
        "[usePriceComparison] API Response data:",
        compareResponse.data,
      );
      console.log(
        "[usePriceComparison] all_prices:",
        compareResponse.data?.all_prices,
      );
      console.log(
        "[usePriceComparison] all_prices length:",
        compareResponse.data?.all_prices?.length,
      );

      setState((prev) => ({
        ...prev,
        data: compareResponse.data,
        loading: false,
      }));
    } catch (error) {
      console.error("Error extracting/comparing:", error);
      setState((prev) => ({
        ...prev,
        loading: false,
        error:
          error instanceof Error ? error.message : "Failed to fetch prices",
      }));
    }
  }

  async function refresh() {
    try {
      const [tab] = await chrome.tabs.query({
        active: true,
        currentWindow: true,
      });
      if (tab?.id) {
        await extractAndCompare(tab.id);
      }
    } catch (error) {
      console.error("Error refreshing:", error);
    }
  }

  return {
    ...state,
    refresh,
  };
}
