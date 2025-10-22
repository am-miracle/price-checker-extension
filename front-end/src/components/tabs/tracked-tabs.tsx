import TrackedItem from "../items/tracked-item";
import { useCompare } from "@/context/compare-context";

const TrackedTabs = () => {
  const { data, isLoading, isError } = useCompare();
  console.log("TrackedTabs context data:", data, { isLoading, isError });
  return (
    <div>
      <TrackedItem />
    </div>
  );
};

export default TrackedTabs;
