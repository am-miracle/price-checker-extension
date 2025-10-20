import React from "react";
import SearchIcon from "./icons/search";
import CloseIcon from "./icons/x";
import TabSection from "./tab-section";

const FloatingCard = () => {
  const [isOpen, setIsOpen] = React.useState(true);

  return (
    <>
      {isOpen && (
        <div className="absolute right-20 top-30 border border-[#E0E0E0] rounded-xl shadow-lg w-[390px]">
          <div className="bg-[#F8F4FF] flex flex-col gap-y-2.5 p-3 rounded-t-xl">
            <div className="flex items-center justify-between">
              <p className="text-[#0D0D0D] font-medium text-2xl leading-9">
                Universal Price Checker
              </p>
              <button
                className="border border-[#666666] h-5 w-5 flex items-center justify-center rounded-sm"
                onClick={() => setIsOpen(false)}
              >
                <CloseIcon size={12} color="#666666" />
              </button>
            </div>

            <div className="border border-[#E0E0E0] flex items-center gap-x-2.5 py-1.5 px-2.5 rounded-lg bg-trasnparent">
              <SearchIcon size={16} color="#666666" />
              <input
                type="text"
                placeholder="Search products..."
                className="w-full outline-none text-sm bg-transparent placeholder:text-gray-400 font-normal leading-5"
              />
            </div>
          </div>
          <TabSection />
        </div>
      )}
    </>
  );
};

export default FloatingCard;
