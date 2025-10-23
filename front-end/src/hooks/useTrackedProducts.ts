import { useState, useEffect, useCallback } from "react";
import type { TrackedProduct, SitePrice } from "@/type/item";
import {
  getTrackedProducts,
  trackProduct,
  untrackProduct,
  isProductTracked,
  getPriceChange,
} from "@/utils/storage";

export function useTrackedProducts() {
  const [products, setProducts] = useState<TrackedProduct[]>([]);
  const [loading, setLoading] = useState(true);

  // Load tracked products
  const loadProducts = useCallback(async () => {
    setLoading(true);
    try {
      const tracked = await getTrackedProducts();
      setProducts(tracked);
    } catch (error) {
      console.error("Error loading tracked products:", error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadProducts();
  }, [loadProducts]);

  // Track a product
  const addTrackedProduct = useCallback(
    async (product: SitePrice, url: string) => {
      try {
        await trackProduct(product, url);
        await loadProducts();
      } catch (error) {
        console.error("Error tracking product:", error);
        throw error;
      }
    },
    [loadProducts]
  );

  // Untrack a product
  const removeTrackedProduct = useCallback(
    async (productId: string) => {
      try {
        await untrackProduct(productId);
        await loadProducts();
      } catch (error) {
        console.error("Error untracking product:", error);
        throw error;
      }
    },
    [loadProducts]
  );

  // Check if product is tracked
  const checkIsTracked = useCallback(async (site: string, url: string) => {
    return await isProductTracked(site, url);
  }, []);

  // Get products with price changes
  const getProductsWithChanges = useCallback(() => {
    return products.map((product) => ({
      product,
      ...getPriceChange(product),
    }));
  }, [products]);

  // Get products with price drops
  const getProductsWithDrops = useCallback(() => {
    return getProductsWithChanges().filter((item) => item.status === "decreased");
  }, [getProductsWithChanges]);

  return {
    products,
    loading,
    addTrackedProduct,
    removeTrackedProduct,
    checkIsTracked,
    getProductsWithChanges,
    getProductsWithDrops,
    refresh: loadProducts,
  };
}
