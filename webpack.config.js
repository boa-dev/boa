const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const {CleanWebpackPlugin} = require("clean-webpack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const webpack = require("webpack");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  entry: "./index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js"
  },
  plugins: [
    new CleanWebpackPlugin(),
    new HtmlWebpackPlugin({
      template: "index.html"
    }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, ".")
    }),
    new CopyWebpackPlugin([
      { from: "./assets/*", to: "." },
      {
        from: "./node_modules/bootstrap/dist/css/bootstrap.min.css",
        to: "./assets"
      }
    ]),
    // Have this example work in Edge which doesn't ship `TextEncoder` or
    // `TextDecoder` at this time.
    new webpack.ProvidePlugin({
      TextDecoder: ["text-encoding", "TextDecoder"],
      TextEncoder: ["text-encoding", "TextEncoder"]
    })
  ],
  mode: "development"
};
