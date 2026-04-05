export default {
  plugins: {
    'postcss-plugin-px2rem': {
      rootValue: 37.5, // 设计稿宽度750px时的配置，可以根据设计稿大小调整此数值
      unitPrecision: 6,
      minPixelValue: 2, // 小于2px的样式不会被转成rem，因为在部分设备上可能会出现小于1px而渲染失
    },
  },
}
