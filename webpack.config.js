const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const webpack = require("webpack");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  experiments: {
    asyncWebAssembly: true,
  },
  entry: {
    app: "./index.js",
    "editor.worker": "monaco-editor/esm/vs/editor/editor.worker.js",
    "json.worker": "monaco-editor/esm/vs/language/json/json.worker",
    "css.worker": "monaco-editor/esm/vs/language/css/css.worker",
    "html.worker": "monaco-editor/esm/vs/language/html/html.worker",
    "ts.worker": "monaco-editor/esm/vs/language/typescript/ts.worker",
  },
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "[name].js",
  },
  plugins: [
    new CleanWebpackPlugin(),
    new HtmlWebpackPlugin({ template: "index.html" }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "./boa_wasm/"),
      outDir: path.resolve(__dirname, "./boa_wasm/pkg/"),
    }),
    new CopyWebpackPlugin({
      patterns: [
        {
          from: "./assets/*",
          to: ".",
        },
        {
          from: "./node_modules/bootstrap/dist/css/bootstrap.min.css",
          to: "./assets",
        },
      ],
    }),
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        use: ["style-loader", "css-loader"],
      },
      {
        test: /\.ttf$/,
        use: ["file-loader"],
      },
    ],
  },
  mode: "development",
};
