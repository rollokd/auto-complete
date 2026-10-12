import { Routes, Route, Outlet, Navigate, useNavigate } from "react-router-dom";
import SidebarLink from "./components/sidebar/link";
import Autocomplete from "./pages/terminal/autocomplete";
import Preferences from "./pages/settings/preferences";
import Integrations from "./pages/settings/integrations";
import Keybindings from "./pages/settings/keybindings";
import Licenses from "./pages/licenses";
import { useContext, useEffect, useRef, useState } from "react";
import Modal from "./components/modal";
import { Event } from "@aws/amazon-q-developer-cli-api-bindings";
import InstallModal from "./components/installs/modal";
import { Glyph } from "./components/svg/logo";
import { getIconFromName } from "./lib/icons";
import { StoreContext } from "./context/zustand";
import { createStore } from "./lib/store";
import ListenerContext from "./context/input";
import {
  useAccessibilityCheck,
  useDotfilesCheck,
  useInputMethodCheck,
} from "./hooks/store";
import { useLocalStateZodDefault } from "./hooks/store/useState";
import { z } from "zod";
import { PRODUCT_NAME } from "./lib/constants";

function App() {
  const store = useRef(createStore()).current;
  const [listening, setListening] = useState<string | null>(null);

  return (
    <StoreContext.Provider value={store}>
      <ListenerContext.Provider value={{ listening, setListening }}>
        <AppLoading />
      </ListenerContext.Provider>
    </StoreContext.Provider>
  );
}

function AppLoading() {
  const store = useContext(StoreContext);
  if (store === null || store().isLoading()) {
    return <div className="w-screen h-screen bg-white dark:bg-zinc-800"></div>;
  } else {
    return <Router />;
  }
}

function useOnboardingComplete() {
  const [onboardingComplete] = useLocalStateZodDefault(
    "desktop.completedOnboarding",
    z.boolean(),
    false,
  );
  return onboardingComplete;
}

function ActiveModal() {
  const onboardingComplete = useOnboardingComplete();

  if (onboardingComplete === false) {
    return (
      <Modal>
        <InstallModal />
      </Modal>
    );
  }

  return null;
}

function Router() {
  const navigate = useNavigate();

  useEffect(() => {
    let unsubscribe: () => void;
    let isStale = false;
    Event.subscribe("dashboard.navigate", (request) => {
      if (
        typeof request === "object" &&
        request !== null &&
        "path" in request &&
        typeof request.path === "string"
      ) {
        navigate(request.path);
      } else {
        console.error("Invalid dashboard.navigate request", request);
      }

      return { unsubscribe: false };
    })?.then((result) => {
      unsubscribe = result.unsubscribe;
      if (isStale) unsubscribe();
    });
    return () => {
      if (unsubscribe) unsubscribe();
      isStale = true;
    };
  }, [navigate]);

  return (
    <>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Navigate to="/autocomplete" replace />} />
          <Route path="autocomplete" element={<Autocomplete />} />
          <Route path="keybindings" element={<Keybindings />} />
          <Route path="integrations" element={<Integrations />} />
          <Route path="preferences" element={<Preferences />} />
          <Route path="licenses" element={<Licenses />} />
          <Route path="*" element={<Navigate to="/autocomplete" replace />} />
        </Route>
      </Routes>
      <ActiveModal />
    </>
  );
}

const navData = [
  {
    type: "link",
    name: "Autocomplete",
    link: "/autocomplete",
  },
  {
    type: "header",
    name: "Settings",
  },
  {
    type: "link",
    name: "Keybindings",
    link: "/keybindings",
  },
  {
    type: "link",
    name: "Integrations",
    link: "/integrations",
  },
  {
    type: "link",
    name: "Preferences",
    link: "/preferences",
  },
];

function Layout() {
  const onboardingComplete = useOnboardingComplete();
  const [accessibilityCheck] = useAccessibilityCheck();
  const [dotfilesCheck] = useDotfilesCheck();
  const [inputMethodCheck] = useInputMethodCheck();
  const error =
    onboardingComplete &&
    (accessibilityCheck === false ||
      dotfilesCheck === false ||
      inputMethodCheck === false);

  return (
    <div className="flex flex-row h-screen w-full overflow-hidden bg-white dark:bg-zinc-800 text-black dark:text-zinc-200">
      <nav className="w-[240px] flex-none h-full flex flex-col items-center gap-1 p-4">
        <div className="flex flex-row items-center gap-2 w-full px-2 pb-3 select-none">
          <Glyph size={24} />
          <span className="text-lg font-semibold tracking-tight">
            {PRODUCT_NAME}
          </span>
        </div>
        {navData.map((item) =>
          item.type === "link" && item.link ? (
            <SidebarLink
              key={item.name}
              path={item.link}
              name={item.name}
              icon={getIconFromName(item.name)}
              error={item.link == "/integrations" ? error : undefined}
            />
          ) : (
            <div
              key={item.name}
              className="pt-4 pl-3 text-sm text-zinc-600 dark:text-zinc-400 w-full rounded-lg flex flex-row items-center font-medium select-none"
            >
              {item.name}
            </div>
          ),
        )}
      </nav>
      <main className="flex flex-col overflow-y-auto p-4 w-full">
        <Outlet />
      </main>
    </div>
  );
}

export default App;
