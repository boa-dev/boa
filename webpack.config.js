const path = require("path");
const fs = require("fs");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const webpack = require("webpack");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const TerserPlugin = require("terser-webpack-plugin");
const MonacoWebpackPlugin = require('monaco-editor-webpack-plugin');

const outdir = path.resolve(__dirname, "./dist");

if (fs.existsSync(outdir)) {
  fs.rmSync(outdir, { recursive: true });
}

module.exports = {
  experiments: {
    asyncWebAssembly: true,
  },
  entry: {
    app: "./index.js"
  },
  output: {
    path: outdir,
    filename: "[name].js",
  },
  plugins: [
    new MonacoWebpackPlugin({
      languages: ["javascript", "typescript"],
      features: [
        "browser",
        "find",
        "inlayHints",
        "documentSymbols",
        "inlineCompletions",
        "parameterHints",
        "snippet",
        "suggest",
        "wordHighlighter",
        "codelens",
        "hover",
        "bracketMatching"
      ]
    }),
    new CleanWebpackPlugin(),
    new HtmlWebpackPlugin({ template: "index.html" }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "./boa_wasm/"),
      outDir: path.resolve(__dirname, "./boa_wasm/pkg/"),
      forceMode: "production",
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
  optimization: {
    minimize: true,
    minimizer: [new TerserPlugin()],
  },
  mode: "development",
};
