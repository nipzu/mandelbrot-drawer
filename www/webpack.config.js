const HtmlWebpackPlugin = require("html-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./src/js/bootstrap.js",
  output: {
    filename: "bundle.js",
    chunkFilename: "[name].bundle.js",
  },
  devtool: 'source-map',
  devServer: {
    contentBase: path.resolve(__dirname, 'dist'),
    watchContentBase: true,
  },
  mode: "development",
  plugins: [
    new HtmlWebpackPlugin({
      filename: './index.html',
      template: './src/html/index.html',
    })
  ], 
  module:{
    rules:[
        {
            test:/\.css$/,
            use:['style-loader','css-loader']
        }
    ]
  },
};
