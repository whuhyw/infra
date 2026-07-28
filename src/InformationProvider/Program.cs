using Microsoft.EntityFrameworkCore;
using InformationProvider.Configuration;
using InformationProvider.Data;
using InformationProvider.Services;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddControllers();
builder.Services.AddDbContext<ElectricityDbContext>(options =>
    options.UseSqlite("Data Source=electricity.db"));
builder.Services.AddOpenApi();
builder.Services.AddSwaggerGen();
builder.Services.AddHttpClient();

var whuApiSection = builder.Configuration.GetSection(WhuApiOptions.SectionName);
builder.Services.Configure<WhuApiOptions>(whuApiSection);

var whuApiOptions = whuApiSection.Get<WhuApiOptions>()
    ?? new WhuApiOptions();

builder.Services.AddSingleton<ISm2CryptoService>(
    _ => new Sm2CryptoService(whuApiOptions.Sm2PublicKey));

builder.Services.AddSingleton<ITokenService, TokenService>();
builder.Services.AddScoped<IWhuApiService, WhuApiService>();
builder.Services.AddScoped<IRoomService, RoomService>();

var app = builder.Build();

if (app.Environment.IsDevelopment())
{
    app.MapOpenApi();
    app.UseSwagger();
    app.UseSwaggerUI();
}

app.UseAuthorization();
app.MapControllers();

using (var scope = app.Services.CreateScope())
{
    var db = scope.ServiceProvider.GetRequiredService<ElectricityDbContext>();
    db.Database.EnsureCreated();
}

app.Run();
