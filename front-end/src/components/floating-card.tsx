"use client";

import React from "react";
import SearchIcon from "./icons/search";
import CloseIcon from "./icons/x";
import TabSection from "./tab-section";
import { useQuery } from "@tanstack/react-query";
import axios from "axios";
import type { CompareResponse } from "@/type/item";
import { CompareProvider } from "@/context/compare-context";


const fetchComparison = async (title: string): Promise<CompareResponse> => {
  const response = await axios.get("https://price-checker-extension.onrender.com/api/compare", {
    params: { title, item: "laptop" },
  });
  return response.data;
};

const FloatingCard = () => {
  const [isOpen, setIsOpen] = React.useState(true);
  const [query, setQuery] = React.useState("laptop");
  const [searchTerm, setSearchTerm] = React.useState("laptop");

  const { data, isLoading, isError } = useQuery({
    queryKey: ["compare", searchTerm],
    queryFn: () => fetchComparison(searchTerm),
    enabled: !!searchTerm,
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!query.trim()) return;
    setSearchTerm(query.trim());
  };

  return (
    <>
      {isOpen && (
        <div className="absolute lg:right-20 top-10 border border-[#E0E0E0] rounded-xl shadow-lg w-full lg:w-[390px] bg-white">
          {/* Header */}
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

            {/* Search Input */}
            <form
              onSubmit={handleSubmit}
              className="border border-[#E0E0E0] flex items-center gap-x-2.5 py-1.5 px-2.5 rounded-lg bg-transparent"
            >
              <SearchIcon size={16} color="#666666" />
              <input
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search products..."
                className="w-full outline-none text-sm bg-transparent placeholder:text-gray-400 font-normal leading-5"
              />
            </form>
          </div>


          {/* Provide compare data via context */}
          <CompareProvider value={{ data, isLoading, isError }}>
            <TabSection />
          </CompareProvider>
        </div>
      )}
    </>
  );
};

export default FloatingCard;
