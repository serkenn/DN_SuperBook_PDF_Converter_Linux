# Test Fixtures

This directory contains test files for integration testing.

## Files

- `sample.pdf` - Simple 1-page PDF for basic testing (auto-generated)
- `multipage.pdf` - 3-page PDF for batch testing (auto-generated)

## Generating Test PDFs

Test PDFs are automatically generated during testing if they don't exist.

To manually generate, run:

```bash
# From project root
cargo test --test cli_integration generate_fixtures -- --ignored
```

Or use ImageMagick:

```bash
# Create a simple test PDF
convert -size 100x100 xc:white -font Helvetica -pointsize 12 \
    -draw "text 10,50 'Test Page 1'" test_page1.png
convert test_page1.png sample.pdf
```

## Test Image Sizes

- Small: 100x100 pixels (quick tests)
- Standard: 2480x3508 pixels (A4 at 300 DPI)
- Large: 4960x7016 pixels (A4 at 600 DPI)
