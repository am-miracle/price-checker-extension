import TrackedItem from "../items/tracked-item";
import { useTrackedProducts } from "@/hooks/useTrackedProducts";

interface TrackedTabsProps {
  searchQuery: string;
}

const TrackedTabs = ({ searchQuery }: TrackedTabsProps) => {
  const { products, loading, removeTrackedProduct } = useTrackedProducts();

  return (
    <div className="h-full">
      <TrackedItem
        products={products}
        loading={loading}
        onRemove={removeTrackedProduct}
        searchQuery={searchQuery}
      />
    </div>
  );
};

export default TrackedTabs;
