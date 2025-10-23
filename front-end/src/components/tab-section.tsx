import AllTabs from "./tabs/all-tabs";
import TrackedTabs from "./tabs/tracked-tabs";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./ui/tabs";
import type { PriceComparisonResult, ProductMatchRequest } from "@/type/item";

interface TabSectionProps {
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

const TabSection = ({ priceData, searchQuery }: TabSectionProps) => {
  return (
    <section className="h-full flex flex-col">
      <Tabs defaultValue="all" className="w-full h-full flex flex-col">
        <TabsList className="border-b border-[#E0E0E0] w-full flex-shrink-0">
          <TabsTrigger value="all">All Prices</TabsTrigger>
          <TabsTrigger value="tracked">Tracked</TabsTrigger>
        </TabsList>

        <TabsContent value="all" className="flex-1 overflow-hidden m-0 p-0">
          <AllTabs priceData={priceData} searchQuery={searchQuery} />
        </TabsContent>

        <TabsContent value="tracked" className="flex-1 overflow-hidden m-0 p-0">
          <TrackedTabs searchQuery={searchQuery} />
        </TabsContent>
      </Tabs>
    </section>
  );
};

export default TabSection;
