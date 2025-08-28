// Function to generate a short code from a URL string
fn generate_short_code(url: &str) -> String {
    // Hash the URL using seahash
    let hash = seahash::hash(url.as_bytes());
    // Format the hash as hexadecimal and take the first 6 characters as the short code
    format!("{:x}", hash)[..6].to_string()
}

fn main() {
    // The original long URL to be shortened
    let long_url = "https://www.example.com/some/long/url";
    // Generate a short code for the long URL
    let short_code = generate_short_code(long_url);
    // Print the shortened URL to the console
    println!("Short URL: http://localhost:8000/{}", short_code);
}
