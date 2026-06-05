// A realistic ASP.NET-style web API controller and service layer.
using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace Acme.Commerce.Api
{
    /// <summary>
    /// Represents a product in the catalogue.
    /// </summary>
    public record Product(int Id, string Name, decimal Price, int Stock);

    /// <summary>
    /// Result returned by service operations.
    /// </summary>
    public record ServiceResult<T>(bool Success, T? Value, string? Error)
    {
        public static ServiceResult<T> Ok(T value) => new(true, value, null);
        public static ServiceResult<T> Fail(string error) => new(false, default, error);
    }

    /// <summary>
    /// Defines the contract for product persistence.
    /// </summary>
    public interface IProductRepository
    {
        Task<Product?> GetByIdAsync(int id, CancellationToken ct = default);
        Task<IReadOnlyList<Product>> ListAsync(int page, int pageSize, CancellationToken ct = default);
        Task<Product> AddAsync(Product product, CancellationToken ct = default);
        Task<bool> UpdateStockAsync(int id, int delta, CancellationToken ct = default);
        Task<bool> DeleteAsync(int id, CancellationToken ct = default);
    }

    /// <summary>
    /// Business logic layer wrapping the product repository.
    /// </summary>
    public class ProductService
    {
        public const int MaxPageSize = 100;
        public const int DefaultPageSize = 20;

        private static readonly TimeSpan CacheExpiry = TimeSpan.FromMinutes(5);

        private readonly IProductRepository _repo;

        public ProductService(IProductRepository repo)
        {
            _repo = repo ?? throw new ArgumentNullException(nameof(repo));
        }

        public async Task<ServiceResult<Product>> GetProductAsync(int id, CancellationToken ct = default)
        {
            var product = await _repo.GetByIdAsync(id, ct);
            return product is null
                ? ServiceResult<Product>.Fail($"Product {id} not found")
                : ServiceResult<Product>.Ok(product);
        }

        public async Task<ServiceResult<IReadOnlyList<Product>>> ListProductsAsync(
            int page = 1, int pageSize = DefaultPageSize, CancellationToken ct = default)
        {
            pageSize = Math.Clamp(pageSize, 1, MaxPageSize);
            var items = await _repo.ListAsync(page, pageSize, ct);
            return ServiceResult<IReadOnlyList<Product>>.Ok(items);
        }

        public async Task<ServiceResult<Product>> CreateProductAsync(
            string name, decimal price, int stock, CancellationToken ct = default)
        {
            if (string.IsNullOrWhiteSpace(name))
                return ServiceResult<Product>.Fail("Name is required");
            if (price < 0)
                return ServiceResult<Product>.Fail("Price must be non-negative");

            var product = new Product(0, name, price, stock);
            var created = await _repo.AddAsync(product, ct);
            return ServiceResult<Product>.Ok(created);
        }

        public async Task<ServiceResult<bool>> AdjustStockAsync(
            int id, int delta, CancellationToken ct = default)
        {
            var ok = await _repo.UpdateStockAsync(id, delta, ct);
            return ok
                ? ServiceResult<bool>.Ok(true)
                : ServiceResult<bool>.Fail($"Failed to adjust stock for product {id}");
        }
    }

    /// <summary>
    /// HTTP controller exposing the product service.
    /// </summary>
    [Route("api/products")]
    [ApiController]
    public class ProductsController
    {
        private readonly ProductService _service;

        public ProductsController(ProductService service)
        {
            _service = service;
        }

        [HttpGet]
        public async Task<IActionResult> GetAll(
            [FromQuery] int page = 1,
            [FromQuery] int pageSize = ProductService.DefaultPageSize,
            CancellationToken ct = default)
        {
            var result = await _service.ListProductsAsync(page, pageSize, ct);
            return result.Success ? Ok(result.Value) : StatusCode(500);
        }

        [HttpGet("{id:int}")]
        public async Task<IActionResult> GetById(int id, CancellationToken ct = default)
        {
            var result = await _service.GetProductAsync(id, ct);
            return result.Success ? Ok(result.Value) : NotFound(result.Error);
        }

        [HttpPost]
        public async Task<IActionResult> Create(
            [FromBody] CreateProductRequest req, CancellationToken ct = default)
        {
            var result = await _service.CreateProductAsync(req.Name, req.Price, req.Stock, ct);
            if (!result.Success)
                return BadRequest(result.Error);
            return CreatedAtAction(nameof(GetById), new { id = result.Value!.Id }, result.Value);
        }
    }

    /// <summary>Request body for product creation.</summary>
    public record CreateProductRequest(string Name, decimal Price, int Stock);

    /// <summary>
    /// Utility helpers.
    /// </summary>
    public static class PriceHelper
    {
        public static readonly decimal VatRate = 0.20m;
        public static readonly string CurrencySymbol = "£";

        public static decimal AddVat(decimal net) => net * (1 + VatRate);
        public static string Format(decimal amount) => $"{CurrencySymbol}{amount:F2}";
    }
}
