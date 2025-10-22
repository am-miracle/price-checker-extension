import { mockPriceData } from "@/data/data";
import LinkIcon from "../icons/link";
import MotorcycleIcon from "../icons/motorcycle";
import { ScrollArea } from "../ui/scroll-area";
import type { Item } from "@/type/item";
import { useState } from "react";
import { useCompare } from "@/context/compare-context";

interface AllItemProps {
  item: Item;
}

const AllItem = () => {
  const { data } = useCompare();
  const allPrices = data?.all_prices;
  console.log("AllTabs context data:", data);
  return (
    <ScrollArea className="h-[50vh] mb-3.5">
      <div className="flex flex-col">
        {allPrices?.map((item, index) => (
          <Item key={index} item={item} />
        ))}
      </div>
    </ScrollArea>
  );
};

const Item = ({ item }: AllItemProps) => {
  const [imageError, setImageError] = useState<boolean>(false);
  return (
    <div className="flex items-center justify-between border-b border-gray-200 p-3">
      <div className="flex items-center gap-x-2">
        {!imageError && item.image ? (
          <img
            src={item.image}
            alt="item"
            className="h-12.5 w-12.5 border object-cover"
            onError={() => setImageError(true)}
          />
        ) : (
          <div className="h-12.5 w-12.5 flex shrink-0 items-center justify-center bg-[#CEBAF8] rounded-lg text-xl font-semibold">
            {item.site.charAt(0)}
          </div>
        )}
        <div className="flex items-end gap-x-3">
          <div className="flex flex-col">
            <p className="text-lg font-normal leading-6.5 text-[#343434]">
              {item.site}
            </p>
            <p className="text-[#121212] font-medium text-xl leading-7">
              {new Intl.NumberFormat("en-NG", {
                style: "currency",
                currency: item.currency || "NGN",
                maximumFractionDigits: 0,
              }).format(Number(item.price))}
            </p>
          </div>
          <div className="flex items-center gap-x-1">
            <MotorcycleIcon />
            <p className="font-normal text-xs leading-5 text-[#7C7C7C]">
              Free shipping
            </p>
          </div>
        </div>
      </div>
      <a
        href={item.link}
        target="_blank"
        className="bg-[#6041B1] p-2.5 rounded-xl flex items-center gap-x-2"
      >
        <span className="text-[#E8E6EC] text-base font-normal leading-6">
          Open
        </span>
        <LinkIcon />
      </a>
    </div>
  );
};

export default AllItem;
