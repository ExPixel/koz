-- CreateEnum
CREATE TYPE "lol_tier" AS ENUM ('IRON', 'BRONZE', 'SILVER', 'GOLD', 'PLATINUM', 'EMERALD', 'DIAMOND', 'MASTER', 'GRANDMASTER', 'CHALLENGER');

-- CreateEnum
CREATE TYPE "lol_ranked_queue" AS ENUM ('SOLO', 'FLEX', 'TWISTED_TREELINE');

-- CreateEnum
CREATE TYPE "lol_region" AS ENUM ('BR', 'EUN', 'EUW', 'JP', 'KR', 'LAN', 'LAS', 'NA', 'OC', 'PH', 'RU', 'SG', 'TH', 'TR', 'TW', 'VN');

-- CreateTable
CREATE TABLE "scheduled_task" (
    "name" VARCHAR(255) NOT NULL,
    "schedule" TEXT NOT NULL,
    "enabled" BOOLEAN NOT NULL DEFAULT true,
    "last_run_at" TIMESTAMPTZ(3),
    "next_run_at" TIMESTAMPTZ(3),

    CONSTRAINT "scheduled_task_pkey" PRIMARY KEY ("name")
);

-- CreateTable
CREATE TABLE "riot_account" (
    "id" BIGSERIAL NOT NULL,
    "puuid" VARCHAR(255) NOT NULL,
    "game_name" VARCHAR(255) NOT NULL,
    "tag_line" VARCHAR(255) NOT NULL,
    "created_at" TIMESTAMPTZ(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "riot_account_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "riot_account_history" (
    "riot_account_id" BIGINT NOT NULL,
    "game_name" TEXT NOT NULL,
    "tag_line" TEXT NOT NULL,
    "updated_at" TIMESTAMPTZ(3) NOT NULL,

    CONSTRAINT "riot_account_history_pkey" PRIMARY KEY ("riot_account_id","updated_at")
);

-- CreateTable
CREATE TABLE "lol_summoner" (
    "id" BIGSERIAL NOT NULL,
    "account_id" TEXT NOT NULL,
    "summoner_id" TEXT NOT NULL,
    "region" "lol_region" NOT NULL,
    "profile_icon_id" INTEGER NOT NULL,
    "revision_date" TIMESTAMPTZ(3) NOT NULL,
    "summoner_level" INTEGER NOT NULL,
    "created_at" TIMESTAMPTZ(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "lol_summoner_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "lol_summoner_profile_history" (
    "lol_summoner_id" BIGINT NOT NULL,
    "profile_icon_id" INTEGER,
    "revision_date" TIMESTAMPTZ(3),
    "summoner_level" INTEGER NOT NULL,
    "updated_at" TIMESTAMPTZ(3) NOT NULL,

    CONSTRAINT "lol_summoner_profile_history_pkey" PRIMARY KEY ("lol_summoner_id","updated_at")
);

-- CreateTable
CREATE TABLE "lol_summoner_rank" (
    "lol_summoner_id" BIGINT NOT NULL,
    "queue_type" "lol_ranked_queue" NOT NULL,
    "tier" "lol_tier" NOT NULL,
    "division" INTEGER NOT NULL,
    "league_points" INTEGER NOT NULL,
    "wins" INTEGER NOT NULL,
    "losses" INTEGER NOT NULL,
    "mini_series_wins" INTEGER,
    "mini_series_losses" INTEGER,
    "mini_series_target" INTEGER,
    "mini_series_progress" TEXT,
    "updated_at" TIMESTAMPTZ(3) NOT NULL,

    CONSTRAINT "lol_summoner_rank_pkey" PRIMARY KEY ("lol_summoner_id","queue_type")
);

-- CreateTable
CREATE TABLE "lol_summoner_rank_history" (
    "lol_summoner_id" BIGINT NOT NULL,
    "queue_type" "lol_ranked_queue" NOT NULL,
    "tier" "lol_tier" NOT NULL,
    "division" INTEGER NOT NULL,
    "league_points" INTEGER NOT NULL,
    "wins" INTEGER NOT NULL,
    "losses" INTEGER NOT NULL,
    "mini_series_wins" INTEGER,
    "mini_series_losses" INTEGER,
    "mini_series_target" INTEGER,
    "mini_series_progress" TEXT,
    "updated_at" TIMESTAMPTZ(3) NOT NULL,

    CONSTRAINT "lol_summoner_rank_history_pkey" PRIMARY KEY ("lol_summoner_id","queue_type","updated_at")
);

-- CreateIndex
CREATE UNIQUE INDEX "riot_account_puuid_key" ON "riot_account"("puuid");

-- CreateIndex
CREATE INDEX "game_name_tag_line_idx" ON "riot_account"("game_name", "tag_line");

-- CreateIndex
CREATE UNIQUE INDEX "lol_summoner_region_summoner_id_key" ON "lol_summoner"("region", "summoner_id");

-- AddForeignKey
ALTER TABLE "riot_account_history" ADD CONSTRAINT "riot_account_history_riot_account_id_fkey" FOREIGN KEY ("riot_account_id") REFERENCES "riot_account"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "lol_summoner_profile_history" ADD CONSTRAINT "lol_summoner_profile_history_lol_summoner_id_fkey" FOREIGN KEY ("lol_summoner_id") REFERENCES "lol_summoner"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "lol_summoner_rank" ADD CONSTRAINT "lol_summoner_rank_lol_summoner_id_fkey" FOREIGN KEY ("lol_summoner_id") REFERENCES "lol_summoner"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "lol_summoner_rank_history" ADD CONSTRAINT "lol_summoner_rank_history_lol_summoner_id_fkey" FOREIGN KEY ("lol_summoner_id") REFERENCES "lol_summoner"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
