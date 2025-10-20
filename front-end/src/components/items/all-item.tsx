import LinkIcon from "../icons/link";
import MotorcycleIcon from "../icons/motorcycle";

const AllItem = () => {
  return (
    <div className="flex items-center justify-between border-b border-gray-200 p-3">
      <div className="flex items-center gap-x-2">
        <div className="h-12.5 w-12.5 flex shrink-0 items-center justify-center bg-[#CEBAF8] rounded-lg text-xl font-semibold">
          A
        </div>
        <div className="flex items-end gap-x-3">
          <div className="flex flex-col">
            <p className="text-lg font-normal leading-6.5 text-[#343434]">
              Amazon
            </p>
            <p className="text-[#121212] font-medium text-xl leading-7">$999</p>
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
        href=""
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
