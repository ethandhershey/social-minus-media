export const dynamic = 'force-static';

export default function manifest() {
    return {
      name: "GatheRound",
      short_name: "GatheRound",
      description:
        "Post activities and meet people in real life.",
      start_url: "/",
      scope: "/",
      display: "standalone",
      orientation: "portrait-primary",
      background_color: "#f5f0e6",
      theme_color: "#7dae8a",
      icons: [
        // {
        //   src: "/icon-192.png",
        //   sizes: "192x192",
        //   type: "image/png",
        //   purpose: "any",
        // },
        {
          src: "/gatheroundicon.svg",
          sizes: "512x512",
          type: "image/svg+xml",
          purpose: "any", //any maskable
        },
      ],
    };
  }