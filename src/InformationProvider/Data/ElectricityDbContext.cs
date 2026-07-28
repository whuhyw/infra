using Microsoft.EntityFrameworkCore;
using InformationProvider.Models;

namespace InformationProvider.Data;

public class ElectricityDbContext : DbContext
{
    public ElectricityDbContext(DbContextOptions<ElectricityDbContext> options) : base(options)
    {
    }

    public DbSet<DailyUsageRecord> DailyUsageRecords => Set<DailyUsageRecord>();

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        modelBuilder.Entity<DailyUsageRecord>(entity =>
        {
            entity.HasIndex(e => new { e.RoomId, e.Date }).IsUnique();
        });
    }
}
