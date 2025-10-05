import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";
import { CategorySpending } from "../../stores/analyticsStore";

interface SpendingBarChartProps {
  categories: CategorySpending[];
}

export function SpendingBarChart({ categories }: SpendingBarChartProps) {
  const data = categories.map((cat) => ({
    name: cat.category_name.length > 15
      ? cat.category_name.substring(0, 15) + "..."
      : cat.category_name,
    amount: cat.amount,
    count: cat.transaction_count,
  }));

  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      return (
        <div className="bg-white dark:bg-gray-800 p-3 rounded-lg border border-gray-200 dark:border-gray-700 shadow-lg">
          <p className="font-semibold text-gray-900 dark:text-white">{label}</p>
          <p className="text-sm text-blue-600 dark:text-blue-400">
            Amount: ${payload[0].value.toFixed(2)}
          </p>
          <p className="text-sm text-gray-600 dark:text-gray-400">
            Transactions: {payload[0].payload.count}
          </p>
        </div>
      );
    }
    return null;
  };

  if (data.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-gray-500 dark:text-gray-400">No data to display</p>
      </div>
    );
  }

  return (
    <ResponsiveContainer width="100%" height={400}>
      <BarChart
        data={data}
        margin={{
          top: 20,
          right: 30,
          left: 20,
          bottom: 60,
        }}
      >
        <CartesianGrid strokeDasharray="3 3" className="stroke-gray-200 dark:stroke-gray-700" />
        <XAxis
          dataKey="name"
          angle={-45}
          textAnchor="end"
          height={80}
          className="text-xs text-gray-600 dark:text-gray-400"
        />
        <YAxis
          className="text-xs text-gray-600 dark:text-gray-400"
          tickFormatter={(value) => `$${value}`}
        />
        <Tooltip content={<CustomTooltip />} />
        <Legend
          wrapperStyle={{
            paddingTop: "20px",
          }}
          formatter={() => (
            <span className="text-sm text-gray-700 dark:text-gray-300">
              Spending Amount
            </span>
          )}
        />
        <Bar dataKey="amount" fill="#3B82F6" radius={[8, 8, 0, 0]} />
      </BarChart>
    </ResponsiveContainer>
  );
}
