import AlertsTabs from "./tabs/alerts-tabs";
import AllTabs from "./tabs/all-tabs";
import TrackedTabs from "./tabs/tracked-tabs";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./ui/tabs";

const TabSection = () => {
  const tabs = [
    {
      value: "all",
      label: "All",
      component: (
        <AllTabs />
      ),
    },
    {
      value: "tracked",
      label: "Tracked",
      component: (
        <TrackedTabs />
      ),
    },
    {
      value: "alerts",
      label: "Alerts",
      component: (
        <AlertsTabs />
      ),
    },
  ];

  return (
    <section>
      <Tabs defaultValue="all" className="w-full">
        <TabsList className="border-y border-[#E0E0E0] w-full">
          {tabs.map((tab) => (
            <TabsTrigger key={tab.value} value={tab.value}>
              {tab.label}
            </TabsTrigger>
          ))}
        </TabsList>

        {tabs.map((tab) => (
          <TabsContent key={tab.value} value={tab.value} className="">
            <div>{tab.component}</div>
          </TabsContent>
        ))}
      </Tabs>
    </section>
  );
};

export default TabSection;
