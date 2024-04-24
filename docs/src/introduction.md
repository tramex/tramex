# Tramex project

Tramex is a project for analyzing and visualizing frames in a 4G network.

Tramex is a project derived from :

- VIGIE
- Amarisoft web interface.

## VIGIE

One of the functionalities of the VIGIE software is to analyze the frames of a 2G/3G network. For more information on this, you can visit <https://hal.science/hal-02141173>.

Tramex can be seen as an upgrade of the VIGIE software because it allows the user to analyze the frames of a 4G network. Note that Tramex does not have all the functionalities of VIGIE.

<details>

<summary>VIGIE Citation</summary>

```bibtex
@article{oyedapo:hal-02141173,
  TITLE = {{VIGIE : A learning tool for cellular air interfaces (GSM, GPRS, UMTS, WiFi)}},
  AUTHOR = {Oyedapo, Olufemi and Martins, Philippe and Lagrange, Xavier},
  URL = {https://hal.science/hal-02141173},
  JOURNAL = {{The IPSI BgD Transactions on Internet Research}},
  HAL_LOCAL_REFERENCE = {1097},
  VOLUME = {1},
  NUMBER = {2},
  PAGES = {65 - 69},
  YEAR = {2005},
  KEYWORDS = {UMTS},
  HAL_ID = {hal-02141173},
  HAL_VERSION = {v1},
}
```

</details>

## Amarisoft web interface

Amarisoft is a software company that provides a 4G LTE software suite. The web interface of Amarisoft lets the user see the frames of a 4G network. This tool uses a web socket to retreive frames of the network.

## Tramex

> Tramex stands for **Tram**e **Ex**ploration.

Tramex uses the same WebSocket to retrieve the frames of the network, but it displays them in a more user-friendly manner, similar to **VIGIE**.
