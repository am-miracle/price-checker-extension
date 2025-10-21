import { mockPriceData } from "@/data/data";
import DeleteIcon from "../icons/delete";
import { ScrollArea } from "../ui/scroll-area";
import type { Item } from "@/type/item";
import { useState } from "react";

interface TrackedItemProps {
  item: Item;
}

const TrackedItem = () => {
  return (
    <ScrollArea className="h-[50vh] mb-3.5">
      <div className="flex flex-col">
        {mockPriceData.map((item,index) => (
          <Item key={index} item={item} />
        ))}
      </div>
    </ScrollArea>
  );
};

const Item = ({ item }: TrackedItemProps) => {
  const [imageError, setImageError] = useState<boolean>(false);
  return (
    <div className="flex gap-x-2.5 border-b border-gray-200 p-3">
      {!imageError && item.image ? (
        <img
          src={item.image}
          alt="item"
          className="h-17.5 w-17.5  border object-cover"
          onError={() => setImageError(true)}
        />
      ) : (
        <div className="h-17.5 w-17.5 flex shrink-0 items-center justify-center bg-[#CEBAF8] rounded-lg text-xl font-semibold">
          {item.site.charAt(0)}
        </div>
      )}
      <div className="flex flex-col gap-y-2">
        <p className="text-black font-normal text-base leading-6">
          {item.title}
        </p>
        <span className="text-xs font-normal text-[#6041B1] leading-5">
          {item.site}
        </span>
        <p className="text-black font-medium text-xl leading-6.5">
          ${item.price.toLocaleString()}
        </p>
        <div className="flex items-center gap-x-2.5 w-full">
          <button className="bg-[#6041B1] w-full rounded-xl p-2.5 text-[#E8E6EC] font-normal text-base leading-6">
            Buy Now
          </button>
          <button className="p-2.5 border border-[#E0E0E0] rounded-xl">
            <DeleteIcon />
          </button>
        </div>
        <p className="text-[#818181] text-xs font-normal leading-5">
          Tracked since Jan 15, 2025
        </p>
      </div>
    </div>
  );
};

export default TrackedItem;
