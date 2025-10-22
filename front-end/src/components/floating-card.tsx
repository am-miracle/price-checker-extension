import React from "react";
import SearchIcon from "./icons/search";
import TabSection from "./tab-section";
import CurrencySelector from "./currency-selector";
import type { PriceComparisonResult, ProductMatchRequest } from "@/type/item";

interface FloatingCardProps {
  priceData: {
    data: PriceComparisonResult | null;
    loading: boolean;
    error: string | null;
    isSupported: boolean;
    currentSite: string | null;
    extractedProduct: ProductMatchRequest | null;
    refresh: () => Promise<void>;
  };
}

const FloatingCard = ({ priceData }: FloatingCardProps) => {
  const [searchQuery, setSearchQuery] = React.useState("");

  const handleCurrencyChange = () => {
    // Trigger price refresh with new currency
    priceData.refresh();
  };

  return (
    <div className="w-full h-full flex flex-col bg-white">
      {/* Header */}
      <div className="bg-[#F8F4FF] flex flex-col gap-y-2.5 p-4 border-b border-[#E0E0E0]">
        <div className="flex items-center justify-between gap-2">
          <div className="flex flex-col flex-1 min-w-0">
            <p className="text-[#0D0D0D] font-semibold text-xl leading-7">
              Price Checker
            </p>
            {priceData.extractedProduct && (
              <p className="text-xs text-[#666666] truncate">
                {priceData.extractedProduct.title}
              </p>
            )}
          </div>
          <div className="flex items-center gap-2 shrink-0">
            <CurrencySelector onCurrencyChange={handleCurrencyChange} />
            {priceData.currentSite && (
              <div className="px-2 py-1 bg-[#6041B1] rounded text-white text-xs font-medium uppercase">
                {priceData.currentSite}
              </div>
            )}
          </div>
        </div>

        <div className="border border-[#E0E0E0] flex items-center gap-x-2.5 py-2 px-3 rounded-lg bg-white">
          <SearchIcon size={16} color="#666666" />
          <input
            type="text"
            placeholder="Filter results..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full outline-none text-sm bg-transparent placeholder:text-gray-400 font-normal"
          />
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden">
        <TabSection priceData={priceData} searchQuery={searchQuery} />
      </div>
    </div>
  );
};

export default FloatingCard;
