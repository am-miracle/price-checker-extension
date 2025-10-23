import { ScrollArea } from "../ui/scroll-area";
import type { TrackedProduct } from "@/type/item";
import { useState, useEffect } from "react";
import LinkIcon from "../icons/link";
import { getPriceChange, getCurrencyPreference } from "@/utils/storage";
import {
  convertCurrency,
  formatPrice,
  getCurrencySymbol,
} from "@/utils/currency";

interface TrackedItemProps {
  products: TrackedProduct[];
  loading: boolean;
  onRemove: (productId: string) => Promise<void>;
  searchQuery: string;
}

const TrackedItem = ({
  products,
  loading,
  onRemove,
  searchQuery,
}: TrackedItemProps) => {
  // Filter products based on search query
  const filteredProducts = searchQuery.trim()
    ? products.filter(
        (product) =>
          product.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
          product.site.toLowerCase().includes(searchQuery.toLowerCase()),
      )
    : products;

  // Loading state
  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-y-3">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-[#6041B1]"></div>
        <p className="text-sm text-[#666666]">Loading tracked products...</p>
      </div>
    );
  }

  // No tracked products
  if (products.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-y-3 p-6">
        <div className="text-[#666666] text-2xl font-semibold">
          No Tracked Products
        </div>
        <p className="text-sm text-[#666666] text-center">
          Start tracking products to monitor price changes
        </p>
        <p className="text-xs text-[#999999] text-center">
          Click the star icon on any product to track it
        </p>
      </div>
    );
  }

  // No search results
  if (filteredProducts.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-y-3 p-6">
        <div className="text-[#666666] text-2xl font-semibold">No Results</div>
        <p className="text-sm text-[#666666] text-center">
          No tracked products match your search
        </p>
      </div>
    );
  }

  return (
    <ScrollArea className="h-full">
      <div className="flex flex-col pb-4">
        {filteredProducts.map((product) => (
          <ProductCard key={product.id} product={product} onRemove={onRemove} />
        ))}
      </div>
    </ScrollArea>
  );
};

interface ProductCardProps {
  product: TrackedProduct;
  onRemove: (productId: string) => Promise<void>;
}

const ProductCard = ({ product, onRemove }: ProductCardProps) => {
  const [imageError, setImageError] = useState(false);
  const [removing, setRemoving] = useState(false);
  const [displayCurrency, setDisplayCurrency] = useState("USD");
  const [displayPrice, setDisplayPrice] = useState(product.current_price);
  const [displayChangeAmount, setDisplayChangeAmount] = useState(0);

  const priceChange = getPriceChange(product);
  const daysSinceTracked = Math.floor(
    (Date.now() - product.tracked_at) / (1000 * 60 * 60 * 24),
  );

  // Get user's preferred currency and convert prices
  useEffect(() => {
    async function loadCurrencyAndConvert() {
      const currency = await getCurrencyPreference();
      setDisplayCurrency(currency);

      // Convert prices
      const converted = await convertCurrency(
        product.current_price,
        product.currency,
        currency,
      );
      setDisplayPrice(converted);

      const convertedChange = await convertCurrency(
        priceChange.change_amount,
        product.currency,
        currency,
      );
      setDisplayChangeAmount(convertedChange);
    }

    loadCurrencyAndConvert();
  }, [product.current_price, product.currency, priceChange.change_amount]);

  const handleRemove = async () => {
    setRemoving(true);
    try {
      await onRemove(product.id);
    } catch (error) {
      console.error("Error removing product:", error);
      setRemoving(false);
    }
  };

  return (
    <div className="border-b border-gray-200 p-3 relative">
      <div className="flex items-start justify-between gap-x-3">
        {/* Product Image & Info */}
        <div className="flex items-start gap-x-2 flex-1 min-w-0">
          {!imageError && product.image ? (
            <img
              src={product.image}
              alt="product"
              className="h-12.5 w-12.5 border object-cover rounded flex-shrink-0"
              onError={() => setImageError(true)}
            />
          ) : (
            <div className="h-12.5 w-12.5 flex shrink-0 items-center justify-center bg-[#CEBAF8] rounded-lg text-xl font-semibold">
              {product.site.charAt(0)}
            </div>
          )}

          <div className="flex flex-col gap-y-1 flex-1 min-w-0">
            <p className="text-sm font-medium text-[#343434] line-clamp-2">
              {product.title}
            </p>
            <p className="text-xs text-[#7C7C7C]">{product.site}</p>

            {/* Current Price */}
            <div className="flex items-center gap-x-2">
              <p className="text-lg font-semibold text-[#121212]">
                {getCurrencySymbol(displayCurrency)}
                {formatPrice(displayPrice)}
              </p>
            </div>

            {/* Price Change Badge */}
            {priceChange.status !== "same" && (
              <div
                className={`inline-flex items-center gap-x-1 text-xs font-medium px-2 py-0.5 rounded self-start ${
                  priceChange.status === "decreased"
                    ? "bg-green-100 text-green-700"
                    : "bg-red-100 text-red-700"
                }`}
              >
                <span>{priceChange.status === "decreased" ? "↓" : "↑"}</span>
                <span>
                  {priceChange.change_percentage.toFixed(1)}% (
                  {getCurrencySymbol(displayCurrency)}
                  {formatPrice(displayChangeAmount)})
                </span>
              </div>
            )}

            <p className="text-[10px] text-[#999999]">
              Tracked{" "}
              {daysSinceTracked === 0 ? "today" : `${daysSinceTracked}d ago`}
            </p>
          </div>
        </div>

        {/* Actions */}
        <div className="flex flex-col gap-y-2 items-end flex-shrink-0">
          <a
            href={product.url}
            target="_blank"
            rel="noopener noreferrer"
            className="bg-[#6041B1] p-2 rounded-lg flex items-center gap-x-1 hover:bg-[#4F3590] transition-colors"
          >
            <span className="text-[#E8E6EC] text-xs font-normal">Open</span>
            <LinkIcon />
          </a>
          <button
            onClick={handleRemove}
            disabled={removing}
            className="text-red-500 text-xs hover:text-red-700 disabled:opacity-50"
          >
            {removing ? "Removing..." : "Untrack"}
          </button>
        </div>
      </div>
    </div>
  );
};

export default TrackedItem;
