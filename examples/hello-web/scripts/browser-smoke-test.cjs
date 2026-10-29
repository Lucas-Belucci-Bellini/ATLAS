const { chromium } = require("playwright");

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  const failures = [];

  page.on("pageerror", (error) => failures.push(`pageerror: ${error.message}`));
  page.on("console", (message) => {
    if (message.type() === "error") failures.push(`console: ${message.text()}`);
  });

  try {
    await page.goto(process.env.ATLAS_BASE_URL || "http://127.0.0.1:8000", {
      waitUntil: "networkidle",
      timeout: 30_000,
    });

    const heading = page.getByRole("heading", { name: "ATLAS — demonstração" });
    await heading.waitFor({ state: "visible", timeout: 15_000 });
    console.log("PASS: WASM inicializou e a página ATLAS foi montada");

    const counter = page.getByRole("button", { name: /Cliques:\\s*0/ });
    await counter.waitFor({ state: "visible" });
    await counter.click();
    await page.getByRole("button", { name: /Cliques:\\s*1/ }).waitFor();
    console.log("PASS: clique atualizou o signal e o texto do contador");

    const nameInput = page.getByLabel("Seu nome");
    await nameInput.fill("Lucas");
    await page.getByText("Olá, Lucas!", { exact: true }).waitFor();
    await nameInput.fill("NEXORA");
    await page.getByText("Olá, NEXORA!", { exact: true }).waitFor();
    console.log("PASS: campo de entrada atualizou a saudação reativa");

    if (failures.length > 0) {
      throw new Error(failures.join("\\n"));
    }
    console.log("ATLAS browser runtime smoke test: PASS");
  } finally {
    await browser.close();
  }
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
