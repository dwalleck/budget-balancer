import { AppLayout } from "./components/layout/AppLayout";
import { TransactionsPage } from "./pages/TransactionsPage";
import { DashboardPage } from "./pages/DashboardPage";
import { DebtPlannerPage } from "./pages/DebtPlannerPage";
import { SpendingAnalysisPage } from "./pages/SpendingAnalysisPage";
import { TrendsPage } from "./pages/TrendsPage";
import { useUIStore } from "./stores/uiStore";
import "./App.css";

function App() {
  const currentPage = useUIStore((state) => state.currentPage);

  const renderPage = () => {
    switch (currentPage) {
      case "transactions":
        return <TransactionsPage />;
      case "dashboard":
        return <DashboardPage />;
      case "debts":
        return <DebtPlannerPage />;
      case "spending":
        return <SpendingAnalysisPage />;
      case "trends":
        return <TrendsPage />;
      case "settings":
        return <div className="p-6">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-4">Settings</h1>
          <p className="text-gray-500 dark:text-gray-400">Coming Soon</p>
        </div>;
      default:
        return <TransactionsPage />;
    }
  };

  return (
    <AppLayout>
      {renderPage()}
    </AppLayout>
  );
}

export default App;
