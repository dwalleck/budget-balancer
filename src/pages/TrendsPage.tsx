import { useEffect, useState } from "react";
import { useAnalyticsStore } from "../stores/analyticsStore";
import { TrendsLineChart } from "../components/visualizations/TrendsLineChart";

export function TrendsPage() {
  const { trends, loading, fetchTrends } = useAnalyticsStore();
  const [interval, setInterval] = useState<"daily" | "weekly" | "monthly">(
    "monthly"
  );
  const [startDate, setStartDate] = useState(
    new Date(new Date().getFullYear(), 0, 1).toISOString().split("T")[0]
  );
  const [endDate, setEndDate] = useState(
    new Date().toISOString().split("T")[0]
  );

  useEffect(() => {
    fetchTrends(startDate, endDate, interval);
  }, [startDate, endDate, interval, fetchTrends]);

  const handleRefresh = () => {
    fetchTrends(startDate, endDate, interval);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
          Spending Trends
        </h1>
        <p className="text-gray-500 dark:text-gray-400">
          Track your spending patterns over time
        </p>
      </div>

      {/* Controls */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 items-end">
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
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Interval
            </label>
            <select
              value={interval}
              onChange={(e) =>
                setInterval(e.target.value as "daily" | "weekly" | "monthly")
              }
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            >
              <option value="daily">Daily</option>
              <option value="weekly">Weekly</option>
              <option value="monthly">Monthly</option>
            </select>
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
          <div className="text-gray-500">Loading trends data...</div>
        </div>
      )}

      {/* Results */}
      {!loading && trends && (
        <>
          {/* Summary */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Total Spending
              </p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">
                ${trends.total_spending.toFixed(2)}
              </p>
            </div>
            <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Average per {interval}
              </p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">
                ${trends.average_per_interval.toFixed(2)}
              </p>
            </div>
            <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Data Points
              </p>
              <p className="text-2xl font-bold text-gray-900 dark:text-white">
                {trends.data_points.length}
              </p>
            </div>
          </div>

          {/* Line Chart */}
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
            <TrendsLineChart
              dataPoints={trends.data_points}
              title="Spending Trends"
            />
          </div>
        </>
      )}
    </div>
  );
}
