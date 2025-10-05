import { useUIStore } from "../../stores/uiStore";
import {
  HomeIcon,
  BanknotesIcon,
  CreditCardIcon,
  ChartBarIcon,
  Cog6ToothIcon,
  ChartPieIcon,
} from "@heroicons/react/24/outline";

interface NavItem {
  name: string;
  icon: React.ComponentType<React.SVGProps<SVGSVGElement>>;
  path: string;
}

function classNames(...classes: string[]) {
  return classes.filter(Boolean).join(" ");
}

export function SidebarNav() {
  const { currentPage, setCurrentPage } = useUIStore();

  const navigation: NavItem[] = [
    { name: "Dashboard", icon: HomeIcon, path: "dashboard" },
    { name: "Transactions", icon: BanknotesIcon, path: "transactions" },
    { name: "Debts", icon: CreditCardIcon, path: "debts" },
    { name: "Spending", icon: ChartPieIcon, path: "spending" },
    { name: "Trends", icon: ChartBarIcon, path: "trends" },
    { name: "Settings", icon: Cog6ToothIcon, path: "settings" },
  ];

  return (
    <nav className="flex flex-1 flex-col">
      <ul role="list" className="flex flex-1 flex-col gap-y-7">
        <li>
          <ul role="list" className="-mx-2 space-y-1">
            {navigation.map((item) => {
              const current = currentPage === item.path;
              return (
                <li key={item.name}>
                  <button
                    onClick={() => setCurrentPage(item.path)}
                    className={classNames(
                      current
                        ? "bg-gray-50 text-indigo-600 dark:bg-white/5 dark:text-white"
                        : "text-gray-700 hover:bg-gray-50 hover:text-indigo-600 dark:text-gray-400 dark:hover:bg-white/5 dark:hover:text-white",
                      "group flex w-full gap-x-3 rounded-md p-2 text-sm font-semibold leading-6"
                    )}
                  >
                    <item.icon
                      aria-hidden="true"
                      className={classNames(
                        current
                          ? "text-indigo-600 dark:text-white"
                          : "text-gray-400 group-hover:text-indigo-600 dark:group-hover:text-white",
                        "h-6 w-6 shrink-0"
                      )}
                    />
                    {item.name}
                  </button>
                </li>
              );
            })}
          </ul>
        </li>
      </ul>
    </nav>
  );
}
