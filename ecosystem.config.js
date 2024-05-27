module.exports = {
  apps: [
    {
      name: "server",
      script: "./target/release/server"
    },
    {
      name: "pallet-stream",
      script: "./target/release/pallet-stream"
    },
    {
      name: "cw721-stream",
      script: "./target/release/cw721-stream"
    }
    // {
    //   name: "schedule",
    //   script: "./target/release/schedule",
    // },
  ]
};
