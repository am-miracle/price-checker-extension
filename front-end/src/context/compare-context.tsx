import React, { createContext, useContext } from "react";
import type { CompareResponse } from "@/type/item";

type CompareContextValue = {
  data: CompareResponse | undefined;
  isLoading: boolean;
  isError: boolean;
};

const CompareContext = createContext<CompareContextValue | undefined>(undefined);

type CompareProviderProps = {
  value: CompareContextValue;
  children: React.ReactNode;
};

export const CompareProvider = ({ value, children }: CompareProviderProps) => {
  return <CompareContext.Provider value={value}>{children}</CompareContext.Provider>;
};

export const useCompare = (): CompareContextValue => {
  const ctx = useContext(CompareContext);
  if (!ctx) {
    throw new Error("useCompare must be used within a CompareProvider");
  }
  return ctx;
};


