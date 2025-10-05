import { useEffect } from "react";
import { useAnalyticsStore } from "../stores/analyticsStore";

export function DashboardPage() {
  const { dashboard, loading, fetchDashboard } = useAnalyticsStore();

  useEffect(() => {
    fetchDashboard("current_month");
  }, [fetchDashboard]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-gray-500">Loading dashboard...</div>
      </div>
    );
  }

  if (!dashboard) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-gray-500">No data available</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
          Dashboard
        </h1>
        <p className="text-gray-500 dark:text-gray-400">
          {dashboard.period.start_date} - {dashboard.period.end_date}
        </p>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* Total Spending */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Total Spending
              </p>
              <p className="text-2xl font-bold text-red-600 dark:text-red-400">
                ${dashboard.total_spending.toFixed(2)}
              </p>
            </div>
            <div className="text-3xl">📉</div>
          </div>
        </div>

        {/* Total Income */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Total Income
              </p>
              <p className="text-2xl font-bold text-green-600 dark:text-green-400">
                ${dashboard.total_income.toFixed(2)}
              </p>
            </div>
            <div className="text-3xl">📈</div>
          </div>
        </div>

        {/* Net */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-500 dark:text-gray-400">Net</p>
              <p
                className={`text-2xl font-bold ${
                  dashboard.net >= 0
                    ? "text-green-600 dark:text-green-400"
                    : "text-red-600 dark:text-red-400"
                }`}
              >
                ${dashboard.net.toFixed(2)}
              </p>
            </div>
            <div className="text-3xl">
              {dashboard.net >= 0 ? "✅" : "⚠️"}
            </div>
          </div>
        </div>
      </div>

      {/* Top Categories */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
        <h2 className="text-xl font-bold mb-4 text-gray-900 dark:text-white">
          Top Spending Categories
        </h2>
        <div className="space-y-3">
          {dashboard.top_categories.map((cat) => (
            <div key={cat.category_id} className="flex items-center gap-3">
              <span className="text-2xl">{cat.category_icon || "📦"}</span>
              <div className="flex-1">
                <div className="flex justify-between mb-1">
                  <span className="text-sm font-medium text-gray-900 dark:text-white">
                    {cat.category_name}
                  </span>
                  <span className="text-sm text-gray-500 dark:text-gray-400">
                    ${cat.amount.toFixed(2)} ({cat.percentage.toFixed(1)}%)
                  </span>
                </div>
                <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                  <div
                    className="bg-blue-600 h-2 rounded-full transition-all"
                    style={{ width: `${cat.percentage}%` }}
                  />
                </div>
              </div>
            </div>
          ))}
          {dashboard.top_categories.length === 0 && (
            <p className="text-gray-500 dark:text-gray-400 text-center py-4">
              No spending data for this period
            </p>
          )}
        </div>
      </div>

      {/* Debt Summary */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
          <h2 className="text-xl font-bold mb-4 text-gray-900 dark:text-white">
            Debt Summary
          </h2>
          <div className="space-y-3">
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">
                Total Debt
              </span>
              <span className="font-semibold text-gray-900 dark:text-white">
                ${dashboard.debt_summary.total_debt.toFixed(2)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">
                Monthly Payment
              </span>
              <span className="font-semibold text-gray-900 dark:text-white">
                ${dashboard.debt_summary.total_monthly_payment.toFixed(2)}
              </span>
            </div>
          </div>
        </div>

        {/* Target Summary */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
          <h2 className="text-xl font-bold mb-4 text-gray-900 dark:text-white">
            Budget Targets
          </h2>
          <div className="space-y-3">
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">
                On Track
              </span>
              <span className="font-semibold text-green-600 dark:text-green-400">
                {dashboard.target_summary.on_track_count}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">
                Over Budget
              </span>
              <span className="font-semibold text-red-600 dark:text-red-400">
                {dashboard.target_summary.over_count}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
