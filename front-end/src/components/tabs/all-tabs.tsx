import Item from "../items/all-item";
import type { PriceComparisonResult, ProductMatchRequest } from "@/type/item";

interface AllTabsProps {
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

const AllTabs = ({ priceData, searchQuery }: AllTabsProps) => {
  return (
    <div className="h-full">
      <Item priceData={priceData} searchQuery={searchQuery} />
    </div>
  );
};

export default AllTabs;
