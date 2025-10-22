import LinkIcon from "../icons/link";
import { ScrollArea } from "../ui/scroll-area";
import type {
  Item,
  PriceComparisonResult,
  ProductMatchRequest,
} from "@/type/item";
import { useState, useMemo, useEffect } from "react";
import { useTrackedProducts } from "@/hooks/useTrackedProducts";
import { isProductTracked } from "@/utils/storage";
import { formatPrice, getCurrencySymbol } from "@/utils/currency";

interface AllItemProps {
  priceData: {
    data: PriceComparisonResult | null;
    loading: boolean;
    error: string | null;
    isSupported: boolean;
    currentSite: string | null;
    extractedProduct: ProductMatchRequest | null;
    refresh: () => Promise<void>;
  };
  searchQuery: string;
}

const AllItem = ({ priceData, searchQuery }: AllItemProps) => {
  const { data, loading, error, isSupported, refresh } = priceData;
  const { addTrackedProduct, refresh: refreshTracked } = useTrackedProducts();

  // Filter items based on search query
  const filteredItems = useMemo(() => {
    if (!data?.all_prices) return [];
    if (!searchQuery.trim()) return data.all_prices;

    const query = searchQuery.toLowerCase();
    return data.all_prices.filter(
      (item) =>
        item.site.toLowerCase().includes(query) ||
        item.title.toLowerCase().includes(query),
    );
  }, [data, searchQuery]);

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-y-3">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-[#6041B1]"></div>
        <p className="text-sm text-[#666666]">Finding best prices...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-y-3 p-6">
        <div className="text-red-500 text-2xl font-semibold">Error</div>
        <p className="text-sm text-[#666666] text-center">{error}</p>
        <button
          onClick={refresh}
          className="bg-[#6041B1] text-white px-4 py-2 rounded-lg text-sm hover:bg-[#4F3590] transition-colors"
        >
          Try Again
        </button>
      </div>
    );
  }

  if (!isSupported) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-y-3 p-6">
        <div className="text-[#666666] text-2xl font-semibold">
          Unsupported Page
        </div>
        <p className="text-sm text-[#666666] text-center font-medium">
          Not a supported product page
        </p>
        <p className="text-xs text-[#999999] text-center">
          Visit a product page on Amazon, eBay, Jumia, or Konga
        </p>
      </div>
    );
  }

  if (!filteredItems.length && data) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-y-3 p-6">
        <div className="text-[#666666] text-2xl font-semibold">No Results</div>
        <p className="text-sm text-[#666666] text-center">
          {searchQuery ? "No results match your search" : "No prices found"}
        </p>
        {data.all_prices.length > 0 && (
          <p className="text-xs text-[#999999]">Try a different search term</p>
        )}
      </div>
    );
  }

  return (
    <ScrollArea className="h-full">
      <div className="flex flex-col pb-4">
        {filteredItems.map((item, index) => (
          <PriceItem
            key={`${item.site}-${index}`}
            item={item}
            isBestDeal={
              data?.best_deal?.site === item.site &&
              data?.best_deal?.price_usd === item.price_usd
            }
            onTrack={addTrackedProduct}
            onTrackComplete={refreshTracked}
          />
        ))}
      </div>
    </ScrollArea>
  );
};

interface PriceItemProps {
  item: Item;
  isBestDeal: boolean;
  onTrack: (product: Item, url: string) => Promise<void>;
  onTrackComplete: () => void;
}

const PriceItem = ({
  item,
  isBestDeal,
  onTrack,
  onTrackComplete,
}: PriceItemProps) => {
  const [imageError, setImageError] = useState<boolean>(false);
  const [isTracked, setIsTracked] = useState<boolean>(false);
  const [tracking, setTracking] = useState<boolean>(false);

  // Check if product is already tracked
  useEffect(() => {
    async function checkTracked() {
      const tracked = await isProductTracked(item.site, item.link);
      setIsTracked(tracked);
    }
    checkTracked();
  }, [item.site, item.link]);

  const handleTrack = async () => {
    if (tracking) return;

    setTracking(true);
    try {
      await onTrack(item, item.link);
      setIsTracked(true);
      onTrackComplete();
    } catch (error) {
      console.error("Error tracking product:", error);
    } finally {
      setTracking(false);
    }
  };

  return (
    <div
      className={`flex items-center justify-between border-b border-gray-200 p-3 ${isBestDeal ? "bg-green-50" : ""}`}
    >
      <div className="flex items-center gap-x-2 min-w-0 flex-1">
        {!imageError && item.image ? (
          <img
            src={item.image}
            alt="item"
            className="h-12.5 w-12.5 border object-cover rounded flex-shrink-0"
            onError={() => setImageError(true)}
          />
        ) : (
          <div className="h-12.5 w-12.5 flex shrink-0 items-center justify-center bg-[#CEBAF8] rounded-lg text-xl font-semibold">
            {item.site.charAt(0)}
          </div>
        )}
        <div className="flex flex-col gap-y-1 min-w-0 flex-1">
          <div className="flex items-center gap-x-2">
            <p className="text-base font-medium leading-6 text-[#343434]">
              {item.site}
            </p>
            {isBestDeal && (
              <span className="bg-green-500 text-white text-[10px] px-1.5 py-0.5 rounded font-medium">
                BEST
              </span>
            )}
          </div>
          <p className="text-[#121212] font-semibold text-lg leading-6">
            {item.target_currency && item.price_converted ? (
              <>
                {getCurrencySymbol(item.target_currency)}
                {formatPrice(Number(item.price_converted))}
              </>
            ) : (
              <>
                {getCurrencySymbol(item.currency)}
                {formatPrice(Number(item.price))}
              </>
            )}
          </p>
          {item.match_confidence !== null &&
            item.match_confidence !== undefined && (
              <p className="text-[10px] text-[#7C7C7C]">
                {item.match_confidence}% match confidence
              </p>
            )}
        </div>
      </div>
      <div className="flex items-center gap-x-2 flex-shrink-0">
        <button
          onClick={handleTrack}
          disabled={isTracked || tracking}
          className={`p-2 rounded-lg transition-colors flex-shrink-0 ${
            isTracked
              ? "text-yellow-500 cursor-not-allowed"
              : "text-gray-400 hover:text-yellow-500"
          }`}
          title={isTracked ? "Already tracking" : "Track this product"}
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-5 w-5"
            fill={isTracked ? "currentColor" : "none"}
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={2}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z"
            />
          </svg>
        </button>
        <a
          href={item.link}
          target="_blank"
          rel="noopener noreferrer"
          className="bg-[#6041B1] p-2.5 rounded-xl flex items-center gap-x-2 hover:bg-[#4F3590] transition-colors"
        >
          <span className="text-[#E8E6EC] text-sm font-normal leading-6">
            Open
          </span>
          <LinkIcon />
        </a>
      </div>
    </div>
  );
};

export default AllItem;
