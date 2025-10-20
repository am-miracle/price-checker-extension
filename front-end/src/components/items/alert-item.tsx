import { alertsMockData } from "@/data/data";
import LinkIcon from "../icons/link";
import CloseIcon from "../icons/x";
import { ScrollArea } from "../ui/scroll-area";
import type { Item } from "@/type/item";

interface AlertItemProps {
  item: Item
}

const AlertItem = () => {
  return (
    <ScrollArea className="h-[50vh] mb-3.5">
      <div className="flex flex-col">
        {alertsMockData.map((item) => (
          <Item key={item.id} item={item}/>
        ))}
      </div>
    </ScrollArea>
  );
};

const Item = ({item}:AlertItemProps) => {
  return (
    <div className="flex justify-between border-b border-gray-200 p-3">
      <div className="flex items-start gap-x-2 w-full">
        <div className="flex items-center gap-x-2">
          <div className="h-2 w-2 rounded-full bg-[#6041B1]" />
          <div className="h-17.5 w-17.5 flex items-center justify-center shrink-0 bg-[#CEBAF8] rounded-lg text-xl font-semibold">
            {item.platform.charAt(0)}
          </div>
        </div>
        <div className="flex flex-col gap-y-2 w-full">
          <div className="flex justify-between">
            <div className="flex flex-col">
              <p className="text-black font-normal text-base leading-6">
                {item.productName}
              </p>
              <p className="text-xs font-normal text-[#6041B1] leading-5">
                {item.platform}
              </p>
            </div>
            <button className="border border-[#666666] h-4 w-4 flex items-center justify-center rounded-sm">
              <CloseIcon size={12} color="#666666" />
            </button>
          </div>
          <div className="flex items-center gap-x-2">
            <span className="line-through text-[#888888] text-xs font-normal">
              {item.oldPrice ? `$${item.oldPrice.toLocaleString()}` : "$299.99"}
            </span>
            <p className="text-black font-medium text-xl leading-6">{item.price}</p>
          </div>
          <a
            href={item.link}
            target="_blank"
            className="bg-[#6041B1] w-full rounded-xl p-2.5 text-[#E8E6EC] font-normal text-base leading-6 flex items-center justify-center gap-x-2"
          >
            <span>View Deal</span>
            <LinkIcon />
          </a>
          <p className="text-[#818181] text-xs font-normal leading-5">
            {item.timeAgo}
          </p>
        </div>
      </div>
    </div>
  );
};

export default AlertItem;
