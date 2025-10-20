import DeleteIcon from "../icons/delete";

const TrackedItem = () => {
  return (
    <div className="flex gap-x-2.5 border-b border-gray-200 p-3">
      <div className="h-17.5 w-17.5 flex items-center justify-center shrink-0 bg-[#CEBAF8] rounded-lg text-xl font-semibold">
        A
      </div>
      <div className="flex flex-col gap-y-2">
        <p className="text-black font-normal text-base leading-6">
          Samsung Galaxy S23 Ultra 256GB Titanium Grey
        </p>
        <span className="text-xs font-normal text-[#6041B1] leading-5">
          Amazon
        </span>
        <p className="text-black font-medium text-xl leading-6.5">$1,998.56</p>
        <div className="flex items-center gap-x-2.5 w-full">
          <button className="bg-[#6041B1] w-full rounded-xl p-2.5 text-[#E8E6EC] font-normal text-base leading-6">
            Buy Now
          </button>
          <button className="p-2.5 border border-[#E0E0E0] rounded-xl">
            <DeleteIcon />
          </button>
        </div>
        <p className="text-[#818181] text-xs font-normal leading-5">Tracked since Jan 15, 2025</p>
      </div>
    </div>
  );
};

export default TrackedItem;
