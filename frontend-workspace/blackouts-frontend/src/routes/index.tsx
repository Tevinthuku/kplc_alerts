import React from "react";
import { createRoot } from "react-dom/client";
import {
  createBrowserRouter,
  RouterProvider,
  Route,
  Link,
} from "react-router-dom";
import SubscribedLocations from "./SubscribedLocations";
import Layout from "../components/Layout";
import MainPage from "./MainPage";

export const router = createBrowserRouter([
  {
    path: "/",
    element: (
      <Layout>
        <MainPage />
      </Layout>
    ),
  },
  {
    path: "/subscribed",
    element: (
      <Layout>
        <SubscribedLocations />
      </Layout>
    ),
  },
]);
