// Function to show loading animation
function showLoadingAnimation() {
    const loadingDiv = document.createElement('div');
    loadingDiv.id = 'loading';
    document.getElementById('report-container').appendChild(loadingDiv);
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
        // Clear containers
        const invoicesContainer = document.getElementById('invoices');
        invoicesContainer.innerHTML = '';
        const totalCard = document.getElementById('total');
        totalCard.innerHTML = '';

        showLoadingAnimation();

        // Get the selected date from the date picker
        const selectedDate = $('#date-picker').data('daterangepicker').startDate.format('D MMM YYYY');

        // Format the selected date to match the format required by the API
        const formattedDate = moment(selectedDate, 'D MMM YYYY').format('YYYY-MM-DD');
        const url = `https://delivr.onrender.com/invoices?organization_id=820117212&date=${formattedDate}`;

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
    const dateInput = $('#date-picker');
    const currentDate = dateInput.data('daterangepicker').startDate;
    const newDate = currentDate.clone().add(days, 'days');

    dateInput.data('daterangepicker').setStartDate(newDate);
    fetchAndDisplayInvoices();
}

function initializeDatePicker() {
    $('#date-picker').daterangepicker({
        locale: {
            format: 'D MMM YYYY'
        },
        singleDatePicker: true,
        showDropdowns: false,
        autoApply: true,
        minYear: 1901,
        maxYear: parseInt(moment().format('YYYY'), 10),
        startDate: moment() // Set the initial date to today
    }, function(start, end, label) {
        fetchAndDisplayInvoices();
    });
}


// Add event listeners to buttons
document.getElementById('left-button').addEventListener('click', () => changeDateBy(-1));
document.getElementById('right-button').addEventListener('click', () => changeDateBy(1));

// Call the function to initialize the date picker when the DOM is ready
$(function() {
    initializeDatePicker();
});
