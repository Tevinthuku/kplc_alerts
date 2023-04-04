import "./App.css";
import { useAuth0 } from "@auth0/auth0-react";
import { useEffect } from "react";
import Login from "./components/Login";
import Layout from "./components/Layout";
import { RouterProvider } from "react-router-dom";
import { router } from "./routes";
import { SWRConfig } from "swr";
import { useAuth } from "./providers/Auth";
import axios from "axios";

const instance = axios.create({});
instance.defaults.headers.common["Content-Type"] = "application/json";

function App() {
  const { isAuthenticated, token } = useAuth();
  console.log(token);
  return (
    <div>
      <div />
      <Layout>
        {token && isAuthenticated ? (
          <SWRConfig
            value={{
              fetcher: async (resource, init) => {
                instance.defaults.headers.common[
                  "Authorization"
                ] = `Bearer ${token}`;
                const apiURL = `/api${resource}`;
                const res = await instance(apiURL);
                return await res.data;
              },
            }}
          >
            <RouterProvider router={router} />
          </SWRConfig>
        ) : (
          <Login />
        )}
      </Layout>
    </div>
  );
}

export default App;
