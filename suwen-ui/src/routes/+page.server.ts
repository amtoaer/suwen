export const load = async () => {
    return {
        shorts: [
            {
                key: "6-29-nuclear-fusion",
                title: "6.29 核聚变",
                image: "https://ipfs.crossbell.io/ipfs/QmULs3E8hFzY5v7ECgrbGbbweZSChR4WzrrriRzGj7tHJ3?img-quality=75&img-format=auto&img-onerror=redirect&img-width=384"
            },
            {
                key: "play-xianwei",
                title: "品了品衔尾：龙之铃",
                image: "https://ipfs.crossbell.io/ipfs/QmSn9VgcT8c3N2aEzq8fzmoVeS8bhNwrEEMEZa1Ni7YCiE?img-quality=75&img-format=auto&img-onerror=redirect&img-width=384"
            },
            {
                key: "finally-mai",
                title: "终于等到不知火舞了！",
                image: "https://ipfs.crossbell.io/ipfs/QmZcQEyTeiFbxMjXJL68vHGgxPx74WJeoMCMefYPGPLGrZ?img-quality=75&img-format=auto&img-onerror=redirect&img-width=384"
            }
        ],
        articles: [
            {
                key: "how-to-use-async-await-in-rust",
                title: "如何在 Rust 中使用 async/await",
                description: "Rust 的 async/await 语法使得编写异步代码变得更加简单和直观。",
                tags: ["Rust", "Async"],
                views: 120,
                comments: 5,
                publishedDate: new Date("2023-10-01"),
                image: "https://ipfs.crossbell.io/ipfs/QmP8gfk1d6uHcGHqsRdcJheYJcpEy8NcnmBt7CwBhFVo35?img-quality=75&img-format=auto&img-onerror=redirect&img-width=1920"
            },
            {
                key: "understanding-svelte-reactiveness",
                title: "深入理解 Svelte 的反应性",
                description: "Svelte 的反应性系统是其核心特性之一，本文将深入探讨其工作原理。",
                tags: ["Svelte", "Reactiveness"],
                views: 200,
                comments: 10,
                publishedDate: new Date("2023-09-15"),
                image: "https://ipfs.crossbell.io/ipfs/Qma9efPHLAo9NT4ywTSm3s9bNpnfziyLkyBp3mab6gYopF?img-quality=75&img-format=auto&img-onerror=redirect&img-width=1920"
            }
        ]
    };
}