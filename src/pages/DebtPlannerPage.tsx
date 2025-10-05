import { useEffect, useState } from "react";
import { useDebtStore } from "../stores/debtStore";
import { Button } from "../components/ui/Button";

export function DebtPlannerPage() {
  const { debts, activePlan, loading, fetchDebts, calculatePayoffPlan } =
    useDebtStore();
  const [strategy, setStrategy] = useState<"avalanche" | "snowball">("avalanche");
  const [monthlyAmount, setMonthlyAmount] = useState("");

  useEffect(() => {
    fetchDebts();
  }, [fetchDebts]);

  const handleCalculate = async () => {
    const amount = parseFloat(monthlyAmount);
    if (amount && amount > 0) {
      await calculatePayoffPlan(strategy, amount);
    }
  };

  const totalDebt = debts.reduce((sum, debt) => sum + debt.balance, 0);
  const totalMinPayment = debts.reduce((sum, debt) => sum + debt.min_payment, 0);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
          Debt Payoff Planner
        </h1>
        <p className="text-gray-500 dark:text-gray-400">
          Calculate your debt payoff strategy
        </p>
      </div>

      {/* Debt Summary */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Total Debt</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-white">
            ${totalDebt.toFixed(2)}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Min Payment</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-white">
            ${totalMinPayment.toFixed(2)}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Debts</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-white">
            {debts.length}
          </p>
        </div>
      </div>

      {/* Debts List */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
        <div className="p-4 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Your Debts
          </h2>
        </div>
        <div className="p-4">
          {debts.length === 0 ? (
            <p className="text-center text-gray-500 dark:text-gray-400 py-8">
              No debts found. Add a debt to get started.
            </p>
          ) : (
            <div className="space-y-3">
              {debts.map((debt) => (
                <div
                  key={debt.id}
                  className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg"
                >
                  <div>
                    <p className="font-medium text-gray-900 dark:text-white">
                      {debt.name}
                    </p>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      {debt.interest_rate}% APR
                    </p>
                  </div>
                  <div className="text-right">
                    <p className="font-semibold text-gray-900 dark:text-white">
                      ${debt.balance.toFixed(2)}
                    </p>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      Min: ${debt.min_payment.toFixed(2)}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Strategy Calculator */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
        <div className="p-4 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Calculate Payoff Plan
          </h2>
        </div>
        <div className="p-4 space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Strategy
            </label>
            <div className="flex gap-4">
              <label className="flex items-center">
                <input
                  type="radio"
                  name="strategy"
                  value="avalanche"
                  checked={strategy === "avalanche"}
                  onChange={(e) =>
                    setStrategy(e.target.value as "avalanche" | "snowball")
                  }
                  className="mr-2"
                />
                <span className="text-gray-900 dark:text-white">
                  Avalanche (Highest Interest First)
                </span>
              </label>
              <label className="flex items-center">
                <input
                  type="radio"
                  name="strategy"
                  value="snowball"
                  checked={strategy === "snowball"}
                  onChange={(e) =>
                    setStrategy(e.target.value as "avalanche" | "snowball")
                  }
                  className="mr-2"
                />
                <span className="text-gray-900 dark:text-white">
                  Snowball (Smallest Balance First)
                </span>
              </label>
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Monthly Payment Amount
            </label>
            <input
              type="number"
              value={monthlyAmount}
              onChange={(e) => setMonthlyAmount(e.target.value)}
              placeholder="Enter amount"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              min={totalMinPayment}
            />
            <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
              Minimum: ${totalMinPayment.toFixed(2)}
            </p>
          </div>

          <Button
            onClick={handleCalculate}
            disabled={loading || !monthlyAmount || parseFloat(monthlyAmount) < totalMinPayment}
          >
            Calculate Payoff Plan
          </Button>
        </div>
      </div>

      {/* Results */}
      {activePlan && (
        <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="p-4 border-b border-gray-200 dark:border-gray-700">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
              Payoff Plan - {activePlan.strategy.charAt(0).toUpperCase() + activePlan.strategy.slice(1)}
            </h2>
          </div>
          <div className="p-4">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
              <div>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  Payoff Date
                </p>
                <p className="text-xl font-bold text-gray-900 dark:text-white">
                  {activePlan.payoff_date}
                </p>
              </div>
              <div>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  Total Interest
                </p>
                <p className="text-xl font-bold text-gray-900 dark:text-white">
                  ${activePlan.total_interest.toFixed(2)}
                </p>
              </div>
              <div>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  Months to Payoff
                </p>
                <p className="text-xl font-bold text-gray-900 dark:text-white">
                  {activePlan.monthly_breakdown.length}
                </p>
              </div>
            </div>

            <div className="mt-4">
              <h3 className="font-semibold mb-4 text-gray-900 dark:text-white">
                Payment Schedule (First 6 Months)
              </h3>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                  <thead className="bg-gray-50 dark:bg-gray-800">
                    <tr>
                      <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                        Month
                      </th>
                      <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                        Payment
                      </th>
                      <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                        Remaining Balance
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
                    {activePlan.monthly_breakdown.slice(0, 6).map((month) => (
                      <tr key={month.month}>
                        <td className="px-4 py-3 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
                          Month {month.month}
                        </td>
                        <td className="px-4 py-3 whitespace-nowrap text-sm text-right text-gray-900 dark:text-white">
                          ${month.total_paid.toFixed(2)}
                        </td>
                        <td className="px-4 py-3 whitespace-nowrap text-sm text-right text-gray-600 dark:text-gray-400">
                          ${month.remaining_balance.toFixed(2)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
