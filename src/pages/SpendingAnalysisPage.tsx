import { useEffect, useState } from "react";
import { useAnalyticsStore } from "../stores/analyticsStore";
import { SpendingPieChart } from "../components/visualizations/SpendingPieChart";
import { SpendingBarChart } from "../components/visualizations/SpendingBarChart";

export function SpendingAnalysisPage() {
  const { spendingByCategory, loading, fetchSpendingByCategory } =
    useAnalyticsStore();
  const [startDate, setStartDate] = useState(
    new Date(new Date().getFullYear(), new Date().getMonth(), 1)
      .toISOString()
      .split("T")[0]
  );
  const [endDate, setEndDate] = useState(
    new Date().toISOString().split("T")[0]
  );

  useEffect(() => {
    fetchSpendingByCategory(startDate, endDate);
  }, [startDate, endDate, fetchSpendingByCategory]);

  const handleRefresh = () => {
    fetchSpendingByCategory(startDate, endDate);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
            Spending Analysis
          </h1>
          <p className="text-gray-500 dark:text-gray-400">
            Analyze your spending by category
          </p>
        </div>
      </div>

      {/* Date Range Selector */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 items-end">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Start Date
            </label>
            <input
              type="date"
              value={startDate}
              onChange={(e) => setStartDate(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              End Date
            </label>
            <input
              type="date"
              value={endDate}
              onChange={(e) => setEndDate(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            />
          </div>
          <div>
            <button
              onClick={handleRefresh}
              className="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
            >
              Refresh
            </button>
          </div>
        </div>
      </div>

      {/* Loading State */}
      {loading && (
        <div className="flex items-center justify-center h-64">
          <div className="text-gray-500">Loading spending data...</div>
        </div>
      )}

      {/* Results */}
      {!loading && spendingByCategory && (
        <>
          {/* Total Spending Card */}
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
            <div className="text-center">
              <p className="text-sm text-gray-500 dark:text-gray-400 mb-2">
                Total Spending
              </p>
              <p className="text-4xl font-bold text-gray-900 dark:text-white">
                ${spendingByCategory.total_spending.toFixed(2)}
              </p>
              <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                {spendingByCategory.period.start_date} to{" "}
                {spendingByCategory.period.end_date}
              </p>
            </div>
          </div>

          {/* Categories Breakdown */}
          <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
            <div className="p-4 border-b border-gray-200 dark:border-gray-700">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                Spending by Category
              </h2>
            </div>
            <div className="p-4">
              {spendingByCategory.categories.length === 0 ? (
                <p className="text-center text-gray-500 dark:text-gray-400 py-8">
                  No spending data for this period
                </p>
              ) : (
                <div className="space-y-4">
                  {spendingByCategory.categories.map((cat) => (
                    <div key={cat.category_id} className="space-y-2">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                          <span className="text-2xl">
                            {cat.category_icon || "📦"}
                          </span>
                          <div>
                            <p className="font-medium text-gray-900 dark:text-white">
                              {cat.category_name}
                            </p>
                            <p className="text-sm text-gray-500 dark:text-gray-400">
                              {cat.transaction_count} transactions
                            </p>
                          </div>
                        </div>
                        <div className="text-right">
                          <p className="font-semibold text-gray-900 dark:text-white">
                            ${cat.amount.toFixed(2)}
                          </p>
                          <p className="text-sm text-gray-500 dark:text-gray-400">
                            {cat.percentage.toFixed(1)}%
                          </p>
                        </div>
                      </div>
                      <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                        <div
                          className="bg-blue-600 h-2 rounded-full transition-all"
                          style={{ width: `${cat.percentage}%` }}
                        />
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Visualizations */}
          {spendingByCategory.categories.length > 0 && (
            <>
              {/* Pie Chart */}
              <div className="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
                <h2 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                  Category Distribution
                </h2>
                <SpendingPieChart categories={spendingByCategory.categories} />
              </div>

              {/* Bar Chart */}
              <div className="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
                <h2 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                  Spending Comparison
                </h2>
                <SpendingBarChart categories={spendingByCategory.categories} />
              </div>
            </>
          )}
        </>
      )}
    </div>
  );
}
