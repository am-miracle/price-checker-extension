import AlertItem from "../items/alert-item"
import { useCompare } from "@/context/compare-context";


const AlertsTabs = () => {
  const { data, isLoading, isError } = useCompare();
  console.log("AlertsTabs context data:", data, { isLoading, isError });
  return (
    <div>
      <AlertItem/>
    </div>
  )
}

export default AlertsTabs