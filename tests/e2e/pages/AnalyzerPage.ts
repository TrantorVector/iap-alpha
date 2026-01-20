import { Page, Locator } from '@playwright/test';

/**
 * Page Object Model for the Analyzer Page
 */
export class AnalyzerPage {
    readonly page: Page;

    // Pane 0: Controls Bar
    readonly companyName: Locator;
    readonly periodTypeQuarterly: Locator;
    readonly periodTypeAnnual: Locator;
    readonly periodCountSelect: Locator;
    readonly refreshButton: Locator;
    readonly closeButton: Locator;

    // Pane 1: Metrics Dashboard
    readonly metricsDashboard: Locator;
    readonly metricsTable: Locator;

    // Pane 2: Document Grid
    readonly documentGrid: Locator;

    // Pane 3: Verdict Form
    readonly verdictInvest: Locator;
    readonly verdictPass: Locator;
    readonly verdictWatchlist: Locator;
    readonly verdictNoThesis: Locator;
    readonly summaryTextarea: Locator;
    readonly saveButton: Locator;
    readonly successToast: Locator;

    // Dialogs
    readonly conflictDialog: Locator;
    readonly confirmCloseDialog: Locator;

    constructor(page: Page) {
        this.page = page;

        // Pane 0: Controls Bar
        this.companyName = page.getByRole('heading', { level: 1 });
        this.periodTypeQuarterly = page.getByRole('tab', { name: /quarterly/i });
        this.periodTypeAnnual = page.getByRole('tab', { name: /annual/i });
        this.periodCountSelect = page.getByRole('combobox', { name: /period count/i });
        this.refreshButton = page.getByRole('button', { name: /refresh/i });
        this.closeButton = page.getByRole('button', { name: /close/i });

        // Pane 1: Metrics Dashboard
        this.metricsDashboard = page.locator('section').filter({ hasText: /key financial metrics/i }).first();
        this.metricsTable = this.metricsDashboard.locator('table').first();

        // Pane 2: Document Grid  
        this.documentGrid = page.locator('section').filter({ hasText: /document repository/i }).first();

        // Pane 3: Verdict Form
        this.verdictInvest = page.getByRole('radio', { name: /invest/i });
        this.verdictPass = page.getByRole('radio', { name: /pass/i });
        this.verdictWatchlist = page.getByRole('radio', { name: /watchlist/i });
        this.verdictNoThesis = page.getByRole('radio', { name: /no thesis/i });
        this.summaryTextarea = page.getByRole('textbox', { name: /summary/i });
        this.saveButton = page.getByRole('button', { name: /save/i });
        this.successToast = page.getByRole('status').filter({ hasText: /success/i });

        // Dialogs
        this.conflictDialog = page.getByRole('dialog', { name: /conflict/i });
        this.confirmCloseDialog = page.getByRole('dialog');
    }

    async goto(companyId: string) {
        await this.page.goto(`/analyzer/${companyId}`);
        await this.page.waitForLoadState('networkidle');
    }

    async selectPeriodType(type: 'quarterly' | 'annual') {
        if (type === 'quarterly') {
            await this.periodTypeQuarterly.click();
        } else {
            await this.periodTypeAnnual.click();
        }
        // Wait for data to reload
        await this.page.waitForLoadState('networkidle');
    }

    async selectPeriodCount(count: number) {
        await this.periodCountSelect.click();
        await this.page.getByRole('option', { name: count.toString() }).click();
        await this.page.waitForLoadState('networkidle');
    }

    async saveVerdict(verdict: 'INVEST' | 'PASS' | 'WATCHLIST' | 'NO_THESIS', summary: string) {
        // Select verdict
        switch (verdict) {
            case 'INVEST':
                await this.verdictInvest.click();
                break;
            case 'PASS':
                await this.verdictPass.click();
                break;
            case 'WATCHLIST':
                await this.verdictWatchlist.click();
                break;
            case 'NO_THESIS':
                await this.verdictNoThesis.click();
                break;
        }

        // Enter summary
        await this.summaryTextarea.fill(summary);

        // Click save
        await this.saveButton.click();
    }

    async waitForMetricsToLoad() {
        await this.metricsTable.waitFor({ state: 'visible' });
    }

    async getMetricValue(metricName: string, columnIndex: number = 1): Promise<string | null> {
        const row = this.page.locator('tr').filter({ hasText: metricName });
        const cell = row.locator('td').nth(columnIndex);
        return await cell.textContent();
    }

    async isDocumentAvailable(docType: string, periodIndex: number = 0): Promise<boolean> {
        const row = this.page.locator('tr').filter({ hasText: docType });
        const cell = row.locator('td').nth(periodIndex + 1); // +1 to skip the first column (doc type name)
        const downloadButton = cell.getByRole('button');
        return await downloadButton.isVisible();
    }
}
