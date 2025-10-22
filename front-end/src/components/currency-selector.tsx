import { useState, useEffect } from "react";
import { getCurrencyPreference, setCurrencyPreference } from "@/utils/storage";

interface Currency {
  code: string;
  symbol: string;
  name: string;
}

interface CurrencySelectorProps {
  onCurrencyChange?: (currency: string) => void;
}

const CURRENCIES: Currency[] = [
  { code: "USD", symbol: "$", name: "US Dollar" },
  { code: "EUR", symbol: "€", name: "Euro" },
  { code: "GBP", symbol: "£", name: "British Pound" },
  { code: "NGN", symbol: "₦", name: "Nigerian Naira" },
  { code: "INR", symbol: "₹", name: "Indian Rupee" },
  { code: "CAD", symbol: "C$", name: "Canadian Dollar" },
  { code: "AUD", symbol: "A$", name: "Australian Dollar" },
  { code: "JPY", symbol: "¥", name: "Japanese Yen" },
];

const CurrencySelector = ({ onCurrencyChange }: CurrencySelectorProps) => {
  const [selectedCurrency, setSelectedCurrency] = useState<string>("USD");
  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    // Load saved preference on mount
    getCurrencyPreference().then(setSelectedCurrency);
  }, []);

  const handleCurrencyChange = async (currencyCode: string) => {
    setSelectedCurrency(currencyCode);
    setIsOpen(false);

    // Save preference
    await setCurrencyPreference(currencyCode);

    // Notify parent component
    if (onCurrencyChange) {
      onCurrencyChange(currencyCode);
    }
  };

  const selectedCurrencyInfo = CURRENCIES.find(c => c.code === selectedCurrency);

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-1 px-2 py-1 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors text-sm font-medium"
        title="Select currency"
      >
        <span>{selectedCurrencyInfo?.symbol || "$"}</span>
        <span>{selectedCurrency}</span>
        <svg
          className={`w-4 h-4 transition-transform ${isOpen ? "rotate-180" : ""}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </button>

      {isOpen && (
        <>
          {/* Backdrop to close dropdown */}
          <div
            className="fixed inset-0 z-10"
            onClick={() => setIsOpen(false)}
          />

          {/* Dropdown menu */}
          <div className="absolute right-0 mt-1 w-48 bg-white border border-gray-200 rounded-md shadow-lg z-20 max-h-60 overflow-y-auto">
            {CURRENCIES.map((currency) => (
              <button
                key={currency.code}
                onClick={() => handleCurrencyChange(currency.code)}
                className={`w-full text-left px-3 py-2 text-sm hover:bg-gray-100 transition-colors flex items-center justify-between ${
                  currency.code === selectedCurrency ? "bg-purple-50 text-[#6041B1]" : ""
                }`}
              >
                <div className="flex items-center gap-2">
                  <span className="font-medium">{currency.symbol}</span>
                  <span>{currency.code}</span>
                </div>
                {currency.code === selectedCurrency && (
                  <svg
                    className="w-4 h-4 text-[#6041B1]"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                  >
                    <path
                      fillRule="evenodd"
                      d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                      clipRule="evenodd"
                    />
                  </svg>
                )}
              </button>
            ))}
          </div>
        </>
      )}
    </div>
  );
};

export default CurrencySelector;
