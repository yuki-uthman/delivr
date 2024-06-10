// Function to show loading animation
function showLoadingAnimation() {
    const loadingDiv = document.createElement('div');
    loadingDiv.id = 'loading';
    document.getElementById('container').appendChild(loadingDiv);
}

// Function to hide loading animation
function hideLoadingAnimation() {
    const loadingDiv = document.getElementById('loading');
    if (loadingDiv) {
        loadingDiv.remove();
    }
}

// Function to fetch and display the data
async function fetchAndDisplayInvoices() {
    try {
        // Clear invoices container
        const invoicesContainer = document.getElementById('invoices');
        invoicesContainer.innerHTML = '';
        const totalCard = document.getElementById('total');
        totalCard.innerHTML = '';

        showLoadingAnimation();

        const selectedDate = document.getElementById('invoice-date').value;
        const url = `https://delivr.onrender.com/invoices?organization_id=820117212&date=${selectedDate}`;

        const response = await fetch(url);
        const data = await response.json();

        hideLoadingAnimation();

        displayInvoices(data);
    } catch (error) {
        hideLoadingAnimation();
        displayInvoices([]);
        console.error('Error fetching invoices:', error);
    }
}

// Function to display the data
function displayInvoices(invoices) {
    const invoicesContainer = document.getElementById('invoices');

    invoices.forEach(invoice => {
        const invoiceDiv = document.createElement('div');
        invoiceDiv.classList.add('invoice');

        const customerName = document.createElement('h2');
        customerName.textContent = `${invoice.customer_name}`;
        invoiceDiv.appendChild(customerName);

        invoice.line_items.forEach(item => {
            const itemDiv = document.createElement('div');
            itemDiv.classList.add('line-item');

            const itemName = document.createElement('p');
            itemName.textContent = `${item.name} x ${item.quantity}pcs = RM${item.item_total.toFixed(2)} (${item.item_profit.toFixed(2)})`;
            itemDiv.appendChild(itemName);

            invoiceDiv.appendChild(itemDiv);
        });

        const totalDiv = document.createElement('div');
        totalDiv.classList.add('invoice-total');
        totalDiv.textContent = `RM${invoice.total.toFixed(2)} (${invoice.profit.toFixed(2)})`;
        invoiceDiv.appendChild(totalDiv);

        invoicesContainer.appendChild(invoiceDiv);
    });


    // Calculate total sales and profit
    let totalSales = 0;
    let totalProfit = 0;

    invoices.forEach(invoice => {
        console.log(invoice);
        invoice.line_items.forEach(item => {
            totalSales += item.item_total;
            totalProfit += item.item_profit;
        });
    });

    // Display total sales and profit
    const totalCard = document.createElement('div');
    totalCard.classList.add('total-card');

    const totalProfitElement = document.createElement('div');
    totalProfitElement.classList.add('total-sales-profit');
    totalProfitElement.textContent = `Total: RM${totalSales.toFixed(2)} (${totalProfit.toFixed(2)})`;
    totalCard.appendChild(totalProfitElement);

    invoicesContainer.appendChild(totalCard);
}

// Function to change date by a specified number of days
function changeDateBy(days) {
    const dateInput = document.getElementById('invoice-date');
    const currentDate = new Date(dateInput.value);
    currentDate.setDate(currentDate.getDate() + days);
    dateInput.valueAsDate = currentDate;
    fetchAndDisplayInvoices();
}

// Add event listener to date picker
document.getElementById('invoice-date').addEventListener('change', fetchAndDisplayInvoices);

// Add event listeners to buttons
document.getElementById('left-button').addEventListener('click', () => changeDateBy(-1));
document.getElementById('right-button').addEventListener('click', () => changeDateBy(1));

// Set default date
let picker = document.getElementById('invoice-date');

var now = new Date();
var day = ("0" + now.getDate()).slice(-2);
var month = ("0" + (now.getMonth() + 1)).slice(-2);
var today = now.getFullYear()+"-"+(month)+"-"+(day) ;

picker.value = today;

fetchAndDisplayInvoices();
