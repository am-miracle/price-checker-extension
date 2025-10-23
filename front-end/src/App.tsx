import "./App.css";
import FloatingCard from "./components/floating-card";
import { usePriceComparison } from "./hooks/usePriceComparison";

function App() {
  const priceComparison = usePriceComparison();

  return (
    <main className="w-[400px] h-[600px] overflow-hidden">
      <FloatingCard priceData={priceComparison} />
    </main>
  );
}

export default App;
