use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use std::{
    fs::File,
    io::{self, Write},
};

#[derive(Debug, Clone, PartialEq)]
enum Screen {
    HouseValue,
    DownPayment,
    HOAFee,
    InterestRate,
    PropertyTax,
    Insurance,
    Maintenance,
    PMI,
    HouseAppreciation,
    LoanTerm,
    ExtraPrincipal,
    Spreadsheet,
    Summary,
}

#[derive(Debug, Clone)]
struct MortgageInputs {
    house_value: String,
    down_payment_percent: String,
    down_payment_amount: String,
    use_percent: bool,
    hoa_fee: String,
    interest_rate: String,
    property_tax_percent: String,
    property_tax_amount: String,
    use_property_tax_percent: bool,
    insurance_percent: String,
    insurance_amount: String,
    use_insurance_percent: bool,
    maintenance_percent: String,
    maintenance_amount: String,
    use_maintenance_percent: bool,
    pmi_percent: String,
    pmi_amount: String,
    use_pmi_percent: bool,
    house_appreciation_rate: String,
    loan_term_years: String,
    extra_principal_payment: String,
}

#[derive(Debug, Clone)]
struct MortgageRow {
    month: u32,
    interest: f64,
    principal: f64,
    extra_principal: f64,
    repair_costs: f64,
    hoa: f64,
    taxes: f64,
    insurance: f64,
    pmi: f64,
    actual_payment: f64,
    cost_of_capital: f64,
    waste_cost: f64,
    cost: f64,
    debt: f64,
    interest_rate: f64,
    house_cost: f64,
    equity: f64,
}

#[derive(Debug, Clone)]
struct MortgageSummary {
    total_interest_paid: f64,
    total_principal_paid: f64,
    total_taxes_paid: f64,
    total_insurance_paid: f64,
    total_maintenance_paid: f64,
    total_pmi_paid: f64,
    total_hoa_paid: f64,
    total_payments: f64,
    total_cost_of_capital: f64,
    total_waste_cost: f64,
    final_house_value: f64,
    final_equity: f64,
    months_to_payoff: u32,
    effective_interest_rate: f64,
}

struct App {
    screen: Screen,
    inputs: MortgageInputs,
    spreadsheet_data: Vec<MortgageRow>,
    table_state: TableState,
    summary: Option<MortgageSummary>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::HouseValue,
            inputs: MortgageInputs {
                house_value: String::new(),
                down_payment_percent: "20".to_string(),
                down_payment_amount: String::new(),
                use_percent: true,
                hoa_fee: "0".to_string(),
                interest_rate: "6.5".to_string(),
                property_tax_percent: "2".to_string(),
                property_tax_amount: String::new(),
                use_property_tax_percent: true,
                insurance_percent: "0.35".to_string(),
                insurance_amount: String::new(),
                use_insurance_percent: true,
                maintenance_percent: "1".to_string(),
                maintenance_amount: String::new(),
                use_maintenance_percent: true,
                pmi_percent: "0.5".to_string(),
                pmi_amount: String::new(),
                use_pmi_percent: true,
                house_appreciation_rate: "3".to_string(),
                loan_term_years: "30".to_string(),
                extra_principal_payment: "0".to_string(),
            },
            spreadsheet_data: Vec::new(),
            table_state: TableState::default(),
            summary: None,
        }
    }
}

impl App {
    fn calculate_mortgage(&mut self) -> Result<()> {
        let house_value: f64 = self.inputs.house_value.parse()?;
        let hoa_monthly: f64 = self.inputs.hoa_fee.parse()?;
        let annual_interest_rate: f64 = self.inputs.interest_rate.parse::<f64>()? / 100.0;
        let monthly_interest_rate = annual_interest_rate / 12.0;
        
        let down_payment = if self.inputs.use_percent {
            let percent: f64 = self.inputs.down_payment_percent.parse()?;
            house_value * (percent / 100.0)
        } else {
            self.inputs.down_payment_amount.parse()?
        };
        
        let loan_amount = house_value - down_payment;
        let down_payment_percent = down_payment / house_value;
        
        // Calculate monthly payment using standard mortgage formula
        let loan_term_years: f64 = self.inputs.loan_term_years.parse()?;
        let num_payments = (loan_term_years * 12.0) as i32;
        let monthly_payment = if monthly_interest_rate > 0.0 {
            loan_amount * (monthly_interest_rate * (1.0 + monthly_interest_rate).powf(num_payments as f64)) 
                / ((1.0 + monthly_interest_rate).powf(num_payments as f64) - 1.0)
        } else {
            loan_amount / num_payments as f64
        };
        
        let extra_principal: f64 = self.inputs.extra_principal_payment.parse()?;
        
        // Property tax calculation
        let (annual_tax_rate, annual_tax_amount) = if self.inputs.use_property_tax_percent {
            let rate = self.inputs.property_tax_percent.parse::<f64>()? / 100.0;
            (rate, 0.0)
        } else {
            let amount = self.inputs.property_tax_amount.parse::<f64>()?;
            (0.0, amount)
        };
        
        // Insurance calculation
        let (annual_insurance_rate, annual_insurance_amount) = if self.inputs.use_insurance_percent {
            let rate = self.inputs.insurance_percent.parse::<f64>()? / 100.0;
            (rate, 0.0)
        } else {
            let amount = self.inputs.insurance_amount.parse::<f64>()?;
            (0.0, amount)
        };
        
        // Maintenance calculation
        let (annual_maintenance_rate, annual_maintenance_amount) = if self.inputs.use_maintenance_percent {
            let rate = self.inputs.maintenance_percent.parse::<f64>()? / 100.0;
            (rate, 0.0)
        } else {
            let amount = self.inputs.maintenance_amount.parse::<f64>()?;
            (0.0, amount)
        };
        
        // PMI calculation (only if down payment < 20%)
        let (pmi_rate, monthly_pmi_amount) = if down_payment_percent < 0.20 {
            if self.inputs.use_pmi_percent {
                let rate = self.inputs.pmi_percent.parse::<f64>()? / 100.0;
                (rate, 0.0)
            } else {
                let amount = self.inputs.pmi_amount.parse::<f64>()?;
                (0.0, amount)
            }
        } else {
            (0.0, 0.0)
        };
        
        // House appreciation rate
        let annual_appreciation_rate = self.inputs.house_appreciation_rate.parse::<f64>()? / 100.0;
        let monthly_appreciation_rate = annual_appreciation_rate / 12.0;
        
        self.spreadsheet_data.clear();
        let mut remaining_balance = loan_amount;
        let mut current_house_value = house_value;
        
        // Summary tracking variables
        let mut total_interest = 0.0;
        let mut total_principal = 0.0;
        let mut total_taxes = 0.0;
        let mut total_insurance = 0.0;
        let mut total_maintenance = 0.0;
        let mut total_pmi = 0.0;
        let mut total_hoa = 0.0;
        let mut total_payments = 0.0;
        let mut total_cost_of_capital = 0.0;
        let mut total_waste_cost = 0.0;
        let mut actual_months = 0;
        
        for month in 1..=360 {
            if remaining_balance <= 0.0 {
                break;
            }
            
            let interest_payment = remaining_balance * monthly_interest_rate;
            let mut principal_payment = monthly_payment - interest_payment;
            
            // Ensure we don't overpay
            if principal_payment + extra_principal > remaining_balance {
                principal_payment = remaining_balance;
            }
            
            // Calculate monthly costs
            current_house_value *= 1.0 + monthly_appreciation_rate;
            
            let monthly_taxes = if annual_tax_rate > 0.0 {
                current_house_value * annual_tax_rate / 12.0
            } else {
                annual_tax_amount / 12.0
            };
            
            let monthly_insurance = if annual_insurance_rate > 0.0 {
                current_house_value * annual_insurance_rate / 12.0
            } else {
                annual_insurance_amount / 12.0
            };
            
            let monthly_pmi = if down_payment_percent < 0.20 && remaining_balance > 0.0 {
                if pmi_rate > 0.0 {
                    remaining_balance * pmi_rate / 12.0
                } else {
                    monthly_pmi_amount
                }
            } else {
                0.0
            };
            
            let monthly_repairs = if annual_maintenance_rate > 0.0 {
                current_house_value * annual_maintenance_rate / 12.0
            } else {
                annual_maintenance_amount / 12.0
            };
            
            let total_payment = interest_payment + principal_payment + extra_principal + 
                               monthly_repairs + hoa_monthly + monthly_taxes + monthly_insurance + monthly_pmi;
            
            // Cost of capital (opportunity cost)
            let equity = current_house_value - remaining_balance;
            let cost_of_capital = equity * annual_interest_rate / 12.0;
            
            // Waste cost = all non-principal payments
            let waste_cost = interest_payment + monthly_repairs + hoa_monthly + monthly_taxes + 
                            monthly_insurance + monthly_pmi + cost_of_capital;
            
            // Total cost
            let total_cost = total_payment - principal_payment - extra_principal + cost_of_capital;
            
            remaining_balance -= principal_payment + extra_principal;
            
            // Update summary totals
            total_interest += interest_payment;
            total_principal += principal_payment + extra_principal;
            total_taxes += monthly_taxes;
            total_insurance += monthly_insurance;
            total_maintenance += monthly_repairs;
            total_pmi += monthly_pmi;
            total_hoa += hoa_monthly;
            total_payments += total_payment;
            total_cost_of_capital += cost_of_capital;
            total_waste_cost += waste_cost;
            actual_months = month;
            
            self.spreadsheet_data.push(MortgageRow {
                month,
                interest: interest_payment,
                principal: principal_payment,
                extra_principal,
                repair_costs: monthly_repairs,
                hoa: hoa_monthly,
                taxes: monthly_taxes,
                insurance: monthly_insurance,
                pmi: monthly_pmi,
                actual_payment: total_payment,
                cost_of_capital,
                waste_cost,
                cost: total_cost,
                debt: remaining_balance,
                interest_rate: annual_interest_rate,
                house_cost: current_house_value,
                equity,
            });
        }
        
        // Calculate summary statistics
        let final_house_value = current_house_value;
        let final_equity = final_house_value;
        let effective_interest_rate = if total_principal > 0.0 {
            (total_interest / total_principal) * (12.0 / actual_months as f64)
        } else {
            0.0
        };
        
        self.summary = Some(MortgageSummary {
            total_interest_paid: total_interest,
            total_principal_paid: total_principal,
            total_taxes_paid: total_taxes,
            total_insurance_paid: total_insurance,
            total_maintenance_paid: total_maintenance,
            total_pmi_paid: total_pmi,
            total_hoa_paid: total_hoa,
            total_payments,
            total_cost_of_capital,
            total_waste_cost,
            final_house_value,
            final_equity,
            months_to_payoff: actual_months,
            effective_interest_rate,
        });
        
        Ok(())
    }
    
    fn export_to_csv(&self, filename: &str) -> Result<()> {
        let mut file = File::create(filename)?;
        
        // Write header
        writeln!(file, "Month,Interest,Principal,Extra Principal,Repair Costs,HOA,Taxes,Insurance,PMI,Actual Payment,Cost of Capital,Waste Cost,Cost,Debt,Interest Rate,House Cost,Equity")?;
        
        // Write data rows
        for row in &self.spreadsheet_data {
            writeln!(
                file,
                "{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.4},{:.2},{:.2}",
                row.month,
                row.interest,
                row.principal,
                row.extra_principal,
                row.repair_costs,
                row.hoa,
                row.taxes,
                row.insurance,
                row.pmi,
                row.actual_payment,
                row.cost_of_capital,
                row.waste_cost,
                row.cost,
                row.debt,
                row.interest_rate,
                row.house_cost,
                row.equity
            )?;
        }
        
        // Write summary if available
        if let Some(summary) = &self.summary {
            writeln!(file)?;
            writeln!(file, "Summary Statistics")?;
            writeln!(file, "Total Interest Paid,{:.2}", summary.total_interest_paid)?;
            writeln!(file, "Total Principal Paid,{:.2}", summary.total_principal_paid)?;
            writeln!(file, "Total Taxes Paid,{:.2}", summary.total_taxes_paid)?;
            writeln!(file, "Total Insurance Paid,{:.2}", summary.total_insurance_paid)?;
            writeln!(file, "Total Maintenance Paid,{:.2}", summary.total_maintenance_paid)?;
            writeln!(file, "Total PMI Paid,{:.2}", summary.total_pmi_paid)?;
            writeln!(file, "Total HOA Paid,{:.2}", summary.total_hoa_paid)?;
            writeln!(file, "Total Payments,{:.2}", summary.total_payments)?;
            writeln!(file, "Total Cost of Capital,{:.2}", summary.total_cost_of_capital)?;
            writeln!(file, "Total Waste Cost,{:.2}", summary.total_waste_cost)?;
            writeln!(file, "Final House Value,{:.2}", summary.final_house_value)?;
            writeln!(file, "Final Equity,{:.2}", summary.final_equity)?;
            writeln!(file, "Months to Payoff,{}", summary.months_to_payoff)?;
            writeln!(file, "Effective Interest Rate,{:.4}", summary.effective_interest_rate)?;
        }
        
        Ok(())
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::default();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.screen {
                Screen::HouseValue => handle_house_value_input(&mut app, key)?,
                Screen::DownPayment => handle_down_payment_input(&mut app, key)?,
                Screen::HOAFee => handle_hoa_input(&mut app, key)?,
                Screen::InterestRate => handle_interest_rate_input(&mut app, key)?,
                Screen::PropertyTax => handle_property_tax_input(&mut app, key)?,
                Screen::Insurance => handle_insurance_input(&mut app, key)?,
                Screen::Maintenance => handle_maintenance_input(&mut app, key)?,
                Screen::PMI => handle_pmi_input(&mut app, key)?,
                Screen::HouseAppreciation => handle_house_appreciation_input(&mut app, key)?,
                Screen::LoanTerm => handle_loan_term_input(&mut app, key)?,
                Screen::ExtraPrincipal => handle_extra_principal_input(&mut app, key)?,
                Screen::Spreadsheet => {
                    if handle_spreadsheet_input(&mut app, key)? {
                        return Ok(());
                    }
                }
                Screen::Summary => {
                    if handle_summary_input(&mut app, key)? {
                        return Ok(());
                    }
                }
            }
        }
    }
}

fn handle_house_value_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char(c) if c.is_numeric() || c == '.' => {
            app.inputs.house_value.push(c);
        }
        KeyCode::Backspace => {
            app.inputs.house_value.pop();
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            if !app.inputs.house_value.is_empty() {
                app.screen = Screen::DownPayment;
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => std::process::exit(0),
        _ => {}
    }
    Ok(())
}

fn handle_down_payment_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Tab => {
            app.inputs.use_percent = !app.inputs.use_percent;
        }
        KeyCode::Char(c) if c.is_numeric() || c == '.' => {
            if app.inputs.use_percent {
                app.inputs.down_payment_percent.push(c);
            } else {
                app.inputs.down_payment_amount.push(c);
            }
        }
        KeyCode::Backspace => {
            if app.inputs.use_percent {
                app.inputs.down_payment_percent.pop();
            } else {
                app.inputs.down_payment_amount.pop();
            }
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            let valid = if app.inputs.use_percent {
                !app.inputs.down_payment_percent.is_empty()
            } else {
                !app.inputs.down_payment_amount.is_empty()
            };
            if valid {
                app.screen = Screen::HOAFee;
            }
        }
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => app.screen = Screen::HouseValue,
        _ => {}
    }
    Ok(())
}

fn handle_hoa_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char(c) if c.is_numeric() || c == '.' => {
            app.inputs.hoa_fee.push(c);
        }
        KeyCode::Backspace => {
            app.inputs.hoa_fee.pop();
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            app.screen = Screen::InterestRate;
        }
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => app.screen = Screen::DownPayment,
        _ => {}
    }
    Ok(())
}

fn handle_interest_rate_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char(c) if c.is_numeric() || c == '.' => {
            app.inputs.interest_rate.push(c);
        }
        KeyCode::Backspace => {
            app.inputs.interest_rate.pop();
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            if !app.inputs.interest_rate.is_empty() {
                app.screen = Screen::PropertyTax;
            }
        }
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => app.screen = Screen::HOAFee,
        _ => {}
    }
    Ok(())
}

fn handle_property_tax_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Tab => {
            app.inputs.use_property_tax_percent = !app.inputs.use_property_tax_percent;
        }
        KeyCode::Char(c) if c.is_numeric() || c == '.' => {
            if app.inputs.use_property_tax_percent {
                app.inputs.property_tax_percent.push(c);
            } else {
                app.inputs.property_tax_amount.push(c);
            }
        }
        KeyCode::Backspace => {
            if app.inputs.use_property_tax_percent {
                app.inputs.property_tax_percent.pop();
            } else {
                app.inputs.property_tax_amount.pop();
            }
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            let valid = if app.inputs.use_property_tax_percent {
                !app.inputs.property_tax_percent.is_empty()
            } else {
                !app.inputs.property_tax_amount.is_empty()
            };
            if valid {
                app.screen = Screen::Insurance;
            }
        }
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => app.screen = Screen::InterestRate,
        _ => {}
    }
    Ok(())
}

fn handle_insurance_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Tab => {
            app.inputs.use_insurance_percent = !app.inputs.use_insurance_percent;
        }
        KeyCode::Char(c) if c.is_numeric() || c == '.' => {
            if app.inputs.use_insurance_percent {
                app.inputs.insurance_percent.push(c);
            } else {
                app.inputs.insurance_amount.push(c);
            }
        }
        KeyCode::Backspace => {
            if app.inputs.use_insurance_percent {
                app.inputs.insurance_percent.pop();
            } else {
                app.inputs.insurance_amount.pop();
            }
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            let valid = if app.inputs.use_insurance_percent {
                !app.inputs.insurance_percent.is_empty()
            } else {
                !app.inputs.insurance_amount.is_empty()
            };
            if valid {
                app.screen = Screen::Maintenance;
            }
        }
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => app.screen = Screen::PropertyTax,
        _ => {}
    }
    Ok(())
}

fn handle_maintenance_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Tab => {
            app.inputs.use_maintenance_percent = !app.inputs.use_maintenance_percent;
        }
        KeyCode::Char(c) if c.is_numeric() || c == '.' => {
            if app.inputs.use_maintenance_percent {
                app.inputs.maintenance_percent.push(c);
            } else {
                app.inputs.maintenance_amount.push(c);
            }
        }
        KeyCode::Backspace => {
            if app.inputs.use_maintenance_percent {
                app.inputs.maintenance_percent.pop();
            } else {
                app.inputs.maintenance_amount.pop();
            }
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            let valid = if app.inputs.use_maintenance_percent {
                !app.inputs.maintenance_percent.is_empty()
            } else {
                !app.inputs.maintenance_amount.is_empty()
            };
            if valid {
                app.screen = Screen::PMI;
            }
        }
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => app.screen = Screen::Insurance,
        _ => {}
    }
    Ok(())
}

fn handle_pmi_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Tab => {
            app.inputs.use_pmi_percent = !app.inputs.use_pmi_percent;
        }
        KeyCode::Char(c) if c.is_numeric() || c == '.' => {
            if app.inputs.use_pmi_percent {
                app.inputs.pmi_percent.push(c);
            } else {
                app.inputs.pmi_amount.push(c);
            }
        }
        KeyCode::Backspace => {
            if app.inputs.use_pmi_percent {
                app.inputs.pmi_percent.pop();
            } else {
                app.inputs.pmi_amount.pop();
            }
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            let valid = if app.inputs.use_pmi_percent {
                !app.inputs.pmi_percent.is_empty()
            } else {
                !app.inputs.pmi_amount.is_empty()
            };
            if valid {
                app.screen = Screen::HouseAppreciation;
            }
        }
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => app.screen = Screen::Maintenance,
        _ => {}
    }
    Ok(())
}

fn handle_house_appreciation_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char(c) if c.is_numeric() || c == '.' || c == '-' => {
            app.inputs.house_appreciation_rate.push(c);
        }
        KeyCode::Backspace => {
            app.inputs.house_appreciation_rate.pop();
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            if !app.inputs.house_appreciation_rate.is_empty() {
                app.screen = Screen::LoanTerm;
            }
        }
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => app.screen = Screen::PMI,
        _ => {}
    }
    Ok(())
}

fn handle_loan_term_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char(c) if c.is_numeric() => {
            app.inputs.loan_term_years.push(c);
        }
        KeyCode::Backspace => {
            app.inputs.loan_term_years.pop();
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            if !app.inputs.loan_term_years.is_empty() {
                app.screen = Screen::ExtraPrincipal;
            }
        }
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => app.screen = Screen::HouseAppreciation,
        _ => {}
    }
    Ok(())
}

fn handle_extra_principal_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char(c) if c.is_numeric() || c == '.' => {
            app.inputs.extra_principal_payment.push(c);
        }
        KeyCode::Backspace => {
            app.inputs.extra_principal_payment.pop();
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            if !app.inputs.extra_principal_payment.is_empty() {
                if let Err(e) = app.calculate_mortgage() {
                    eprintln!("Error calculating mortgage: {}", e);
                } else {
                    app.screen = Screen::Spreadsheet;
                    app.table_state.select(Some(0));
                }
            }
        }
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => app.screen = Screen::LoanTerm,
        _ => {}
    }
    Ok(())
}

fn handle_spreadsheet_input(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
        KeyCode::Esc => {
            app.screen = Screen::ExtraPrincipal;
            Ok(false)
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.screen = Screen::Summary;
            Ok(false)
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            let filename = "mortgage_spreadsheet.csv";
            match app.export_to_csv(filename) {
                Ok(_) => {
                    eprintln!("Exported to {}", filename);
                }
                Err(e) => {
                    eprintln!("Error exporting to CSV: {}", e);
                }
            }
            Ok(false)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let current = app.table_state.selected().unwrap_or(0);
            if current < app.spreadsheet_data.len() - 1 {
                app.table_state.select(Some(current + 1));
            }
            Ok(false)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let current = app.table_state.selected().unwrap_or(0);
            if current > 0 {
                app.table_state.select(Some(current - 1));
            }
            Ok(false)
        }
        KeyCode::PageDown | KeyCode::Char('d') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            let current = app.table_state.selected().unwrap_or(0);
            let new_pos = (current + 10).min(app.spreadsheet_data.len() - 1);
            app.table_state.select(Some(new_pos));
            Ok(false)
        }
        KeyCode::PageUp | KeyCode::Char('u') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            let current = app.table_state.selected().unwrap_or(0);
            let new_pos = current.saturating_sub(10);
            app.table_state.select(Some(new_pos));
            Ok(false)
        }
        KeyCode::Char('g') => {
            app.table_state.select(Some(0));
            Ok(false)
        }
        KeyCode::Char('G') => {
            if !app.spreadsheet_data.is_empty() {
                app.table_state.select(Some(app.spreadsheet_data.len() - 1));
            }
            Ok(false)
        }
        KeyCode::Char('h') | KeyCode::Left => {
            app.screen = Screen::ExtraPrincipal;
            Ok(false)
        }
        _ => Ok(false),
    }
}

fn handle_summary_input(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
            app.screen = Screen::Spreadsheet;
            Ok(false)
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            let filename = "mortgage_analysis.csv";
            match app.export_to_csv(filename) {
                Ok(_) => {
                    // In a real app, we'd show a success message
                    eprintln!("Exported to {}", filename);
                }
                Err(e) => {
                    eprintln!("Error exporting to CSV: {}", e);
                }
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::HouseValue => render_house_value_screen(f, app),
        Screen::DownPayment => render_down_payment_screen(f, app),
        Screen::HOAFee => render_hoa_screen(f, app),
        Screen::InterestRate => render_interest_rate_screen(f, app),
        Screen::PropertyTax => render_property_tax_screen(f, app),
        Screen::Insurance => render_insurance_screen(f, app),
        Screen::Maintenance => render_maintenance_screen(f, app),
        Screen::PMI => render_pmi_screen(f, app),
        Screen::HouseAppreciation => render_house_appreciation_screen(f, app),
        Screen::LoanTerm => render_loan_term_screen(f, app),
        Screen::ExtraPrincipal => render_extra_principal_screen(f, app),
        Screen::Spreadsheet => render_spreadsheet_screen(f, app),
        Screen::Summary => render_summary_screen(f, app),
    }
}

fn render_house_value_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("Home Buyer Calculator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("What is the value of the house you're considering buying?");
    
    let input = Paragraph::new(format!("${}", app.inputs.house_value))
        .style(Style::default().fg(Color::Yellow))
        .block(input_block);
    f.render_widget(input, chunks[1]);

    let help = Paragraph::new("Enter/l/→: continue | Esc/q: exit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

fn render_down_payment_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(7),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("Home Buyer Calculator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let percent_value = format!("{}%", app.inputs.down_payment_percent);
    let amount_value = format!("${}", app.inputs.down_payment_amount);
    
    let percent_option = if app.inputs.use_percent {
        format!("▶ Percentage: {}", percent_value)
    } else {
        format!("  Percentage: {}", percent_value)
    };
    
    let amount_option = if !app.inputs.use_percent {
        format!("▶ Dollar Amount: {}", amount_value)
    } else {
        format!("  Dollar Amount: {}", amount_value)
    };

    let options_text = vec![
        Line::from(percent_option).style(if app.inputs.use_percent { 
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) 
        } else { 
            Style::default().fg(Color::DarkGray) 
        }),
        Line::from(amount_option).style(if !app.inputs.use_percent { 
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) 
        } else { 
            Style::default().fg(Color::DarkGray) 
        }),
    ];

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Down Payment - Press Tab to switch between options");
    
    let input = Paragraph::new(options_text)
        .block(input_block);
    f.render_widget(input, chunks[1]);

    let help = Paragraph::new("Tab: toggle between % and $ | Enter/l/→: continue | Esc/h/←: back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

fn render_hoa_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("Home Buyer Calculator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("What is the monthly HOA fee? (0 if none)");
    
    let input = Paragraph::new(format!("${}", app.inputs.hoa_fee))
        .style(Style::default().fg(Color::Yellow))
        .block(input_block);
    f.render_widget(input, chunks[1]);

    let help = Paragraph::new("Enter/l/→: continue | Esc/h/←: back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

fn render_interest_rate_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("Home Buyer Calculator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("What is your expected interest rate? (%)");
    
    let input = Paragraph::new(format!("{}%", app.inputs.interest_rate))
        .style(Style::default().fg(Color::Yellow))
        .block(input_block);
    f.render_widget(input, chunks[1]);

    let help = Paragraph::new("Enter/l/→: calculate | Esc/h/←: back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

fn render_spreadsheet_screen(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let header_cells = vec![
        "Month", "Interest", "Principal", "Extra Principal", "Repair Costs", 
        "HOA", "Taxes", "Insurance", "PMI", "Actual Payment", 
        "Cost of Capital", "Waste Cost", "Cost", "Debt", 
        "Interest Rate", "House Cost", "Equity"
    ];
    let header = Row::new(header_cells)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .height(1);

    let rows = app.spreadsheet_data.iter().map(|row| {
        let cells = vec![
            Cell::from(row.month.to_string()),
            Cell::from(format!("${:.0}", row.interest)),
            Cell::from(format!("${:.0}", row.principal)),
            Cell::from(format!("${:.0}", row.extra_principal)),
            Cell::from(format!("${:.0}", row.repair_costs)),
            Cell::from(format!("${:.0}", row.hoa)),
            Cell::from(format!("${:.0}", row.taxes)),
            Cell::from(format!("${:.0}", row.insurance)),
            Cell::from(format!("${:.0}", row.pmi)),
            Cell::from(format!("${:.0}", row.actual_payment)),
            Cell::from(format!("${:.0}", row.cost_of_capital)),
            Cell::from(format!("${:.0}", row.waste_cost)),
            Cell::from(format!("${:.0}", row.cost)),
            Cell::from(format!("${:.0}", row.debt)),
            Cell::from(format!("{:.2}%", row.interest_rate * 100.0)),
            Cell::from(format!("${:.0}", row.house_cost)),
            Cell::from(format!("${:.0}", row.equity)),
        ];
        Row::new(cells).height(1)
    });

    let widths = [
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(15),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(14),
        Constraint::Length(15),
        Constraint::Length(11),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Length(13),
        Constraint::Length(12),
        Constraint::Length(12),
    ];
    
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Mortgage Spreadsheet"))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(">> ");

    f.render_stateful_widget(table, chunks[0], &mut app.table_state);

    let help = Paragraph::new("j/k or ↑/↓: navigate | g/G: top/bottom | s: summary | e: export CSV | h/←: back | q: quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[1]);
}

fn render_property_tax_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(7),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("Home Buyer Calculator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let percent_value = format!("{}%", app.inputs.property_tax_percent);
    let amount_value = format!("${}", app.inputs.property_tax_amount);
    
    let percent_option = if app.inputs.use_property_tax_percent {
        format!("▶ Annual Percentage of Home Value: {}", percent_value)
    } else {
        format!("  Annual Percentage of Home Value: {}", percent_value)
    };
    
    let amount_option = if !app.inputs.use_property_tax_percent {
        format!("▶ Fixed Annual Amount: {}", amount_value)
    } else {
        format!("  Fixed Annual Amount: {}", amount_value)
    };

    let options_text = vec![
        Line::from(percent_option).style(if app.inputs.use_property_tax_percent { 
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) 
        } else { 
            Style::default().fg(Color::DarkGray) 
        }),
        Line::from(amount_option).style(if !app.inputs.use_property_tax_percent { 
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) 
        } else { 
            Style::default().fg(Color::DarkGray) 
        }),
    ];

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Property Tax - Press Tab to switch between options");
    
    let input = Paragraph::new(options_text)
        .block(input_block);
    f.render_widget(input, chunks[1]);

    let help = Paragraph::new("Tab: toggle between % and $ | Enter/l/→: continue | Esc/h/←: back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

fn render_insurance_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(7),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("Home Buyer Calculator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let percent_value = format!("{}%", app.inputs.insurance_percent);
    let amount_value = format!("${}", app.inputs.insurance_amount);
    
    let percent_option = if app.inputs.use_insurance_percent {
        format!("▶ Annual Percentage of Home Value: {}", percent_value)
    } else {
        format!("  Annual Percentage of Home Value: {}", percent_value)
    };
    
    let amount_option = if !app.inputs.use_insurance_percent {
        format!("▶ Fixed Annual Amount: {}", amount_value)
    } else {
        format!("  Fixed Annual Amount: {}", amount_value)
    };

    let options_text = vec![
        Line::from(percent_option).style(if app.inputs.use_insurance_percent { 
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) 
        } else { 
            Style::default().fg(Color::DarkGray) 
        }),
        Line::from(amount_option).style(if !app.inputs.use_insurance_percent { 
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) 
        } else { 
            Style::default().fg(Color::DarkGray) 
        }),
    ];

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Homeowners Insurance - Press Tab to switch between options");
    
    let input = Paragraph::new(options_text)
        .block(input_block);
    f.render_widget(input, chunks[1]);

    let help = Paragraph::new("Tab: toggle between % and $ | Enter/l/→: continue | Esc/h/←: back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

fn render_maintenance_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(7),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("Home Buyer Calculator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let percent_value = format!("{}%", app.inputs.maintenance_percent);
    let amount_value = format!("${}", app.inputs.maintenance_amount);
    
    let percent_option = if app.inputs.use_maintenance_percent {
        format!("▶ Annual Percentage of Home Value: {}", percent_value)
    } else {
        format!("  Annual Percentage of Home Value: {}", percent_value)
    };
    
    let amount_option = if !app.inputs.use_maintenance_percent {
        format!("▶ Fixed Annual Amount: {}", amount_value)
    } else {
        format!("  Fixed Annual Amount: {}", amount_value)
    };

    let options_text = vec![
        Line::from(percent_option).style(if app.inputs.use_maintenance_percent { 
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) 
        } else { 
            Style::default().fg(Color::DarkGray) 
        }),
        Line::from(amount_option).style(if !app.inputs.use_maintenance_percent { 
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) 
        } else { 
            Style::default().fg(Color::DarkGray) 
        }),
    ];

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Maintenance/Repair Costs - Press Tab to switch between options");
    
    let input = Paragraph::new(options_text)
        .block(input_block);
    f.render_widget(input, chunks[1]);

    let help = Paragraph::new("Tab: toggle between % and $ | Enter/l/→: continue | Esc/h/←: back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

fn render_pmi_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(8),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("Home Buyer Calculator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let percent_value = format!("{}%", app.inputs.pmi_percent);
    let amount_value = format!("${}", app.inputs.pmi_amount);
    
    let percent_option = if app.inputs.use_pmi_percent {
        format!("▶ Annual Percentage of Loan Balance: {}", percent_value)
    } else {
        format!("  Annual Percentage of Loan Balance: {}", percent_value)
    };
    
    let amount_option = if !app.inputs.use_pmi_percent {
        format!("▶ Fixed Monthly Amount: {}", amount_value)
    } else {
        format!("  Fixed Monthly Amount: {}", amount_value)
    };

    let down_payment_note = if let Ok(house_value) = app.inputs.house_value.parse::<f64>() {
        let down_payment_result = if app.inputs.use_percent {
            app.inputs.down_payment_percent.parse::<f64>().map(|p| p / 100.0)
        } else {
            app.inputs.down_payment_amount.parse::<f64>().map(|a| a / house_value)
        };
        
        if let Ok(down_payment) = down_payment_result {
            if down_payment < 0.20 {
                "(Required: down payment < 20%)"
            } else {
                "(Not required: down payment >= 20%)"
            }
        } else {
            ""
        }
    } else {
        ""
    };

    let mut options_text = vec![
        Line::from(percent_option).style(if app.inputs.use_pmi_percent { 
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) 
        } else { 
            Style::default().fg(Color::DarkGray) 
        }),
        Line::from(amount_option).style(if !app.inputs.use_pmi_percent { 
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) 
        } else { 
            Style::default().fg(Color::DarkGray) 
        }),
    ];
    
    if !down_payment_note.is_empty() {
        options_text.push(Line::from(""));
        options_text.push(Line::from(down_payment_note).style(Style::default().fg(Color::Cyan)));
    }

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("PMI - Private Mortgage Insurance - Press Tab to switch between options");
    
    let input = Paragraph::new(options_text)
        .block(input_block);
    f.render_widget(input, chunks[1]);

    let help = Paragraph::new("Tab: toggle between % and $ | Enter/l/→: continue | Esc/h/←: back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

fn render_house_appreciation_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("Home Buyer Calculator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Annual House Appreciation/Inflation Rate (%) - can be negative");
    
    let input = Paragraph::new(format!("{}%", app.inputs.house_appreciation_rate))
        .style(Style::default().fg(Color::Yellow))
        .block(input_block);
    f.render_widget(input, chunks[1]);

    let help = Paragraph::new("Enter/l/→: calculate | Esc/h/←: back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}
fn render_loan_term_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("Home Buyer Calculator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Loan Term (years) - common values: 15, 20, 30");
    
    let input = Paragraph::new(format!("{} years", app.inputs.loan_term_years))
        .style(Style::default().fg(Color::Yellow))
        .block(input_block);
    f.render_widget(input, chunks[1]);

    let help = Paragraph::new("Enter/l/→: continue | Esc/h/←: back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

fn render_extra_principal_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("Home Buyer Calculator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Extra Monthly Principal Payment (optional)");
    
    let input = Paragraph::new(format!("${}", app.inputs.extra_principal_payment))
        .style(Style::default().fg(Color::Yellow))
        .block(input_block);
    f.render_widget(input, chunks[1]);

    let help = Paragraph::new("Enter/l/→: calculate | Esc/h/←: back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

fn render_summary_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Paragraph::new("Mortgage Summary")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    if let Some(summary) = &app.summary {
        let text = vec![
            Line::from(vec![
                Span::styled("Total Payments: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("${:.0}", summary.total_payments)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Principal Paid: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("${:.0}", summary.total_principal_paid), Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Interest Paid: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("${:.0}", summary.total_interest_paid), Style::default().fg(Color::Red)),
            ]),
            Line::from(vec![
                Span::styled("Property Taxes: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("${:.0}", summary.total_taxes_paid), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Insurance: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("${:.0}", summary.total_insurance_paid), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Maintenance: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("${:.0}", summary.total_maintenance_paid), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("PMI: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("${:.0}", summary.total_pmi_paid), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("HOA Fees: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("${:.0}", summary.total_hoa_paid), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Cost of Capital: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("${:.0}", summary.total_cost_of_capital), Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::styled("Total Waste Cost: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("${:.0}", summary.total_waste_cost), Style::default().fg(Color::Red)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Final House Value: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("${:.0}", summary.final_house_value), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Final Equity: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("${:.0}", summary.final_equity), Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Months to Payoff: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{} ({:.1} years)", summary.months_to_payoff, summary.months_to_payoff as f64 / 12.0)),
            ]),
            Line::from(vec![
                Span::styled("Effective Interest Rate: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:.2}%", summary.effective_interest_rate * 100.0)),
            ]),
        ];

        let summary_widget = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Financial Summary"))
            .alignment(Alignment::Left);
        
        f.render_widget(summary_widget, chunks[1]);
    }

    let help = Paragraph::new("e: export to CSV | h/←: back to spreadsheet | q: quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[2]);
}